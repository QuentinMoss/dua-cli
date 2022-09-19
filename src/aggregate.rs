use crate::{crossdev, InodeFilter, WalkOptions, WalkResult};
use anyhow::Result;
use filesize::PathExt;
use owo_colors::{AnsiColors as Color, OwoColorize};
use std::{io, path::Path};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

/// Throttle access to an optional `io::Write` to the specified `Duration`
#[derive(Debug)]
struct ThrottleWriter<W> {
    out: Option<W>,
    trigger: Arc<AtomicBool>,
}

impl<W> ThrottleWriter<W>
where
    W: io::Write,
{
    fn new(out: Option<W>, duration: Duration) -> Self {
        let writer = Self {
            out,
            trigger: Default::default(),
        };

        if writer.out.is_some() {
            let trigger = Arc::downgrade(&writer.trigger);
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(1));
                while let Some(t) = trigger.upgrade() {
                    t.store(true, Ordering::Relaxed);
                    thread::sleep(duration);
                }
            });
        }

        writer
    }

    fn throttled<F>(&mut self, f: F)
    where
        F: FnOnce(&mut W),
    {
        if self.trigger.swap(false, Ordering::Relaxed) {
            self.unthrottled(f);
        }
    }

    fn unthrottled<F>(&mut self, f: F)
    where
        F: FnOnce(&mut W),
    {
        if let Some(ref mut out) = self.out {
            f(out);
        }
    }
}

/// Aggregate the given `paths` and write information about them to `out` in a human-readable format.
/// If `compute_total` is set, it will write an additional line with the total size across all given `paths`.
/// If `sort_by_size_in_bytes` is set, we will sort all sizes (ascending) before outputting them.
pub fn aggregate(
    mut out: impl io::Write,
    err: Option<impl io::Write>,
    walk_options: WalkOptions,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<(WalkResult, Statistics)> {
    let mut res = WalkResult::default();
    let mut stats = Statistics {
        smallest_file_in_bytes: u128::max_value(),
        ..Default::default()
    };
    let mut total = 0;
    let mut num_roots = 0;
    let mut aggregates = Vec::new();
    let mut inodes = InodeFilter::default();
    let mut progress = ThrottleWriter::new(err, Duration::from_millis(100));

    for path in paths.into_iter() {
        num_roots += 1;
        let mut num_bytes = 0u128;
        let mut num_errors = 0u64;
        let device_id = match crossdev::init(path.as_ref()) {
            Ok(id) => id,
            Err(_) => {
                num_errors += 1;
                res.num_errors += 1;
                aggregates.push((path.as_ref().to_owned(), num_bytes, num_errors));
                continue;
            }
        };
        for entry in walk_options.iter_from_path(path.as_ref()) {
            stats.entries_traversed += 1;
            progress.throttled(|out| {
                write!(out, "Enumerating {} entries\r", stats.entries_traversed).ok();
            });
            match entry {
                Ok(entry) => {
                    let file_size = match entry.client_state {
                        Some(Ok(ref m))
                            if !m.is_dir()
                                && (walk_options.count_hard_links || inodes.add(m))
                                && (walk_options.cross_filesystems
                                    || crossdev::is_same_device(device_id, m)) =>
                        {
                            if walk_options.apparent_size {
                                m.len()
                            } else {
                                entry.path().size_on_disk_fast(m).unwrap_or_else(|_| {
                                    num_errors += 1;
                                    0
                                })
                            }
                        }
                        Some(Ok(_)) => 0,
                        Some(Err(_)) => {
                            num_errors += 1;
                            0
                        }
                        None => 0, // ignore directory
                    } as u128;
                    stats.largest_file_in_bytes = stats.largest_file_in_bytes.max(file_size);
                    stats.smallest_file_in_bytes = stats.smallest_file_in_bytes.min(file_size);
                    num_bytes += file_size;
                }
                Err(_) => num_errors += 1,
            }
        }
        progress.unthrottled(|out| {
            write!(out, "\x1b[2K\r").ok();
        });

        if sort_by_size_in_bytes {
            aggregates.push((path.as_ref().to_owned(), num_bytes, num_errors));
        } else {
            output_colored_path(
                &mut out,
                &walk_options,
                &path,
                num_bytes,
                num_errors,
                path_color_of(&path),
            )?;
        }
        total += num_bytes;
        res.num_errors += num_errors;
    }

    if stats.entries_traversed == 0 {
        stats.smallest_file_in_bytes = 0;
    }

    if sort_by_size_in_bytes {
        aggregates.sort_by_key(|&(_, num_bytes, _)| num_bytes);
        for (path, num_bytes, num_errors) in aggregates.into_iter() {
            output_colored_path(
                &mut out,
                &walk_options,
                &path,
                num_bytes,
                num_errors,
                path_color_of(&path),
            )?;
        }
    }

    if num_roots > 1 && compute_total {
        output_colored_path(
            &mut out,
            &walk_options,
            Path::new("total"),
            total,
            res.num_errors,
            None,
        )?;
    }
    Ok((res, stats))
}

fn path_color_of(path: impl AsRef<Path>) -> Option<Color> {
    (!path.as_ref().is_file()).then(|| Color::Cyan)
}

fn output_colored_path(
    out: &mut impl io::Write,
    options: &WalkOptions,
    path: impl AsRef<Path>,
    num_bytes: u128,
    num_errors: u64,
    path_color: Option<Color>,
) -> std::result::Result<(), io::Error> {
    let size = options.byte_format.display(num_bytes).to_string();
    let size = size.green();
    let size_width = options.byte_format.width();
    let path = path.as_ref().display();

    let errors = (num_errors != 0)
        .then(|| {
            let plural_s = if num_errors > 1 { "s" } else { "" };
            format!("  <{num_errors} IO Error{plural_s}>")
        })
        .unwrap_or_default();

    if let Some(color) = path_color {
        writeln!(out, "{size:>size_width$} {}{errors}", path.color(color))
    } else {
        writeln!(out, "{size:>size_width$} {path}{errors}")
    }
}

/// Statistics obtained during a filesystem walk
#[derive(Default, Debug)]
pub struct Statistics {
    /// The amount of entries we have seen during filesystem traversal
    pub entries_traversed: u64,
    /// The size of the smallest file encountered in bytes
    pub smallest_file_in_bytes: u128,
    /// The size of the largest file encountered in bytes
    pub largest_file_in_bytes: u128,
}
