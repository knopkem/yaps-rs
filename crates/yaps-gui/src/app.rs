//! Main iced application state and logic.

use std::path::PathBuf;

use iced::widget::{
    button, checkbox, column, container, horizontal_rule, pick_list, progress_bar, row,
    scrollable, text, text_input, Column,
};
use iced::{Center, Element, Fill, Task};

use crate::messages::{
    ConflictChoice, DuplicateChoice, FolderTarget, Message, OperationChoice, ReportData,
    SortingResult,
};

/// Application state.
pub struct App {
    // Paths
    source: String,
    target: String,

    // Patterns
    folder_pattern: String,
    file_pattern: String,

    // Options
    operation: OperationChoice,
    conflict: ConflictChoice,
    duplicate: DuplicateChoice,
    recursive: bool,
    dry_run: bool,
    detect_duplicates: bool,

    // State
    phase: Phase,
}

impl Default for App {
    fn default() -> Self {
        Self {
            source: String::new(),
            target: String::new(),
            folder_pattern: "{year}/{month}-{month_long}".to_string(),
            file_pattern: "{day}-{month_short}-{hour}{minute}{second}-{filename}".to_string(),
            operation: OperationChoice::default(),
            conflict: ConflictChoice::default(),
            duplicate: DuplicateChoice::default(),
            recursive: true,
            dry_run: false,
            detect_duplicates: true,
            phase: Phase::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
enum Phase {
    #[default]
    Setup,
    Running,
    Done(ReportData),
    Error(String),
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::SourceChanged(s) => app.source = s,
        Message::TargetChanged(s) => app.target = s,
        Message::BrowseSource => return open_folder_dialog(FolderTarget::Source),
        Message::BrowseTarget => return open_folder_dialog(FolderTarget::Target),
        Message::FolderSelected(target, path) => {
            if let Some(p) = path {
                let s = p.to_string_lossy().to_string();
                match target {
                    FolderTarget::Source => app.source = s,
                    FolderTarget::Target => app.target = s,
                }
            }
        }

        Message::FolderPatternChanged(s) => app.folder_pattern = s,
        Message::FilePatternChanged(s) => app.file_pattern = s,

        Message::OperationSelected(op) => app.operation = op,
        Message::ConflictSelected(c) => app.conflict = c,
        Message::DuplicateSelected(d) => app.duplicate = d,
        Message::ToggleRecursive(v) => app.recursive = v,
        Message::ToggleDryRun(v) => app.dry_run = v,
        Message::ToggleDedup(v) => app.detect_duplicates = v,

        Message::StartSorting => {
            app.phase = Phase::Running;
            return run_sorting(app);
        }
        Message::SortingComplete(result) => match result {
            SortingResult::Success(data) => app.phase = Phase::Done(data),
            SortingResult::Error(msg) => app.phase = Phase::Error(msg),
        },
        Message::Reset => app.phase = Phase::Setup,
    }
    Task::none()
}

fn view(app: &App) -> Element<'_, Message> {
    let content = match &app.phase {
        Phase::Setup => view_setup(app),
        Phase::Running => view_running(),
        Phase::Done(data) => view_report(data),
        Phase::Error(msg) => view_error(msg),
    };

    container(content).width(Fill).height(Fill).padding(20).into()
}

fn view_setup(app: &App) -> Element<'_, Message> {
    let title = text("YAPS-rs — Photo Sorter").size(28);

    let source_row = row![
        text("Source:").width(100),
        text_input("Path to photos...", &app.source)
            .on_input(Message::SourceChanged)
            .width(Fill),
        button("Browse").on_press(Message::BrowseSource),
    ]
    .spacing(8)
    .align_y(Center);

    let target_row = row![
        text("Target:").width(100),
        text_input("Output directory...", &app.target)
            .on_input(Message::TargetChanged)
            .width(Fill),
        button("Browse").on_press(Message::BrowseTarget),
    ]
    .spacing(8)
    .align_y(Center);

    let patterns = view_patterns(app);
    let options = view_options(app);

    let can_start = !app.source.is_empty() && !app.target.is_empty();
    let start_btn = if can_start {
        button(text("Start Sorting").size(16))
            .on_press(Message::StartSorting)
            .padding([8, 24])
    } else {
        button(text("Start Sorting").size(16)).padding([8, 24])
    };

    scrollable(
        column![
            title,
            horizontal_rule(1),
            source_row,
            target_row,
            horizontal_rule(1),
            patterns,
            horizontal_rule(1),
            options,
            horizontal_rule(1),
            start_btn,
        ]
        .spacing(12)
        .padding(10)
        .width(Fill),
    )
    .into()
}

fn view_patterns(app: &App) -> Element<'_, Message> {
    let folder_pat = row![
        text("Folder pattern:").width(100),
        text_input("{year}/{month}-{month_long}", &app.folder_pattern)
            .on_input(Message::FolderPatternChanged)
            .width(Fill),
    ]
    .spacing(8)
    .align_y(Center);

    let file_pat = row![
        text("File pattern:").width(100),
        text_input(
            "{day}-{month_short}-{hour}{minute}{second}-{filename}",
            &app.file_pattern,
        )
        .on_input(Message::FilePatternChanged)
        .width(Fill),
    ]
    .spacing(8)
    .align_y(Center);

    let hint = text(
        "Tags: {year} {month} {month_short} {month_long} {day} {day_short} {day_long} \
         {hour} {minute} {second} {week} {make} {model} {lens} {iso} {aperture} \
         {shutter} {focal} {width} {height} {orientation} {media_type} {filename} {ext} \
         {gps_lat} {gps_lon}",
    )
    .size(12);

    column![text("Patterns").size(18), folder_pat, file_pat, hint,]
        .spacing(8)
        .into()
}

fn view_options(app: &App) -> Element<'_, Message> {
    let dropdowns = row![
        column![
            text("Operation:").size(14),
            pick_list(
                OperationChoice::ALL,
                Some(app.operation),
                Message::OperationSelected,
            ),
        ]
        .spacing(4),
        column![
            text("On conflict:").size(14),
            pick_list(
                ConflictChoice::ALL,
                Some(app.conflict),
                Message::ConflictSelected,
            ),
        ]
        .spacing(4),
        column![
            text("Duplicates:").size(14),
            pick_list(
                DuplicateChoice::ALL,
                Some(app.duplicate),
                Message::DuplicateSelected,
            ),
        ]
        .spacing(4),
    ]
    .spacing(16);

    let checkboxes = row![
        checkbox("Recursive", app.recursive).on_toggle(Message::ToggleRecursive),
        checkbox("Dry run", app.dry_run).on_toggle(Message::ToggleDryRun),
        checkbox("Detect duplicates", app.detect_duplicates).on_toggle(Message::ToggleDedup),
    ]
    .spacing(16);

    column![text("Options").size(18), dropdowns, checkboxes,]
        .spacing(8)
        .into()
}

fn view_running<'a>() -> Element<'a, Message> {
    column![
        text("Sorting in progress...").size(24),
        progress_bar(0.0..=1.0, 0.5).width(Fill),
        text("Please wait while files are being organized.").size(14),
    ]
    .spacing(20)
    .padding(40)
    .align_x(Center)
    .width(Fill)
    .into()
}

fn view_report<'a>(data: &ReportData) -> Element<'a, Message> {
    let heading = if data.files_failed == 0 {
        text("✓ Sorting complete!").size(24)
    } else {
        text("⚠ Sorting complete with errors").size(24)
    };

    let stats: Column<Message> = column![
        report_line("Time elapsed", &format!("{:.2}s", data.elapsed_secs)),
        report_line("Files found", &data.files_total.to_string()),
        report_line("With EXIF", &data.files_with_exif.to_string()),
        report_line("Without EXIF", &data.files_without_exif.to_string()),
        report_line("Processed", &data.files_processed.to_string()),
        report_line("Duplicates", &data.duplicates.to_string()),
        report_line("Conflicts", &data.conflicts.to_string()),
        report_line("Skipped", &data.files_skipped.to_string()),
        report_line("Failed", &data.files_failed.to_string()),
    ]
    .spacing(4);

    column![
        heading,
        horizontal_rule(1),
        stats,
        horizontal_rule(1),
        button(text("Sort More"))
            .on_press(Message::Reset)
            .padding([8, 24]),
    ]
    .spacing(16)
    .padding(20)
    .width(Fill)
    .into()
}

fn view_error<'a>(msg: &str) -> Element<'a, Message> {
    column![
        text("✗ Error").size(24),
        text(msg.to_string()).size(14),
        button(text("Back"))
            .on_press(Message::Reset)
            .padding([8, 24]),
    ]
    .spacing(16)
    .padding(20)
    .width(Fill)
    .into()
}

fn open_folder_dialog(target: FolderTarget) -> Task<Message> {
    Task::perform(
        async move {
            // No native dialog — user types the path manually
            None::<PathBuf>
        },
        move |path| Message::FolderSelected(target, path),
    )
}

fn run_sorting(app: &App) -> Task<Message> {
    let config = yaps_core::Config {
        source: PathBuf::from(&app.source),
        target: PathBuf::from(&app.target),
        folder_pattern: app.folder_pattern.clone(),
        file_pattern: app.file_pattern.clone(),
        file_operation: app.operation.to_file_operation(),
        conflict_strategy: app.conflict.to_strategy(),
        duplicate_strategy: app.duplicate.to_strategy(),
        detect_duplicates: app.detect_duplicates,
        recursive: app.recursive,
        dry_run: app.dry_run,
        ..yaps_core::Config::default()
    };

    Task::perform(
        async move {
            let (tx, rx) = tokio::sync::oneshot::channel();
            std::thread::spawn(move || {
                let result = yaps_core::ops::organizer::Organizer::run(&config, None);
                let _ = tx.send(result);
            });
            rx.await
        },
        |result| match result {
            Ok(Ok(report)) => {
                Message::SortingComplete(SortingResult::Success(ReportData::from(&report)))
            }
            Ok(Err(e)) => Message::SortingComplete(SortingResult::Error(e.to_string())),
            Err(e) => {
                Message::SortingComplete(SortingResult::Error(format!("Task error: {e}")))
            }
        },
    )
}

fn report_line<'a>(label: &str, value: &str) -> Element<'a, Message> {
    row![text(format!("{label}:")).width(120), text(value.to_string()),]
        .spacing(8)
        .into()
}

/// Run the iced application.
pub fn run() -> iced::Result {
    iced::application("YAPS-rs — Photo Sorter", update, view)
        .window_size(iced::Size::new(700.0, 600.0))
        .centered()
        .run()
}
