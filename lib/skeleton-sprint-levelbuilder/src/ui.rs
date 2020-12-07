mod error_modal;
mod note_modal;
mod style;
mod widgets;

pub use self::error_modal::{
    ErrorModal,
    ErrorModalMessage,
};
use self::{
    note_modal::{
        NoteModal,
        NoteModalMessage,
    },
    style::DarkTheme,
    widgets::{
        Board,
        ToolBar,
    },
};
use crate::AppError;
use iced_core::{
    Length,
    Point,
    Rectangle,
};
use sks::format::LevelNumber;
use std::sync::Arc;

/// Assumes it CAN be translated infallibly. TODO: Do i make this return an option?
pub fn get_relative_position(bounds: &Rectangle, pos: &Point) -> Point {
    Point::new(pos.x - bounds.x, pos.y - bounds.y)
}

#[derive(Debug)]
pub enum AppState {
    Builder,

    NoteModal,
    ErrorModal,
    ExportModal,
}

impl AppState {
    pub fn new() -> Self {
        AppState::Builder
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum ExportModalState {
    ChooseFormat,
    Lbl,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddBlock { index: usize, block: sks::Block },
    ImportLevel(String),
    SetLevelNumber(Option<LevelNumber>),
    SetDark(bool),
    SetGrid(bool),
    ChangeActiveBlock { block: Option<sks::Block> },
    OpenExportModal,

    NoteModalMessage(NoteModalMessage),
    ErrorModalMessage(ErrorModalMessage),

    ChangeExportModalState(ExportModalState),
    ExportModalLblTextInputChanged(String),

    CloseExportModal,

    RequestLevelImport,

    RequestSaveLblExport,

    Nop,
}

pub struct UiApp {
    pub level: sks::Level,
    active_block: Option<sks::Block>,

    grid: bool,

    iced_block_map: crate::IcedBlockMap,
    background_image: iced_native::image::Handle,
    trash_bin_image: iced_native::image::Handle,

    board_state: widgets::board::State,
    tool_bar_state: widgets::tool_bar::State,
    import_button_state: iced::button::State,
    export_button_state: iced::button::State,

    app_state: AppState,

    note_modal: NoteModal,
    error_modal: ErrorModal,

    export_modal_state: ExportModalState,
    export_modal_close_button_state: iced::widget::button::State,
    export_modal_export_type_lbl_button_state: iced::widget::button::State,
    export_modal_export_lbl_text_input_state: iced::widget::text_input::State,
    export_modal_export_lbl_text_input_content: String,
}

impl UiApp {
    pub fn new(iced_block_map: crate::IcedBlockMap) -> Self {
        let background_image = iced::image::Handle::from_memory(crate::M0_DATA.into());
        let trash_bin_image = iced::image::Handle::from_memory(crate::TRASH_BIN_DATA.into());

        Self {
            level: sks::Level::new(),
            active_block: None,

            grid: true,

            iced_block_map,
            background_image,
            trash_bin_image,

            board_state: widgets::board::State::new(),
            tool_bar_state: widgets::tool_bar::State::new(),
            import_button_state: iced::button::State::new(),
            export_button_state: iced::button::State::new(),

            app_state: AppState::Builder,

            note_modal: NoteModal::new(),
            error_modal: ErrorModal::new(),

            export_modal_state: ExportModalState::ChooseFormat,
            export_modal_close_button_state: iced::widget::button::State::new(),
            export_modal_export_type_lbl_button_state: iced::widget::button::State::new(),

            export_modal_export_lbl_text_input_state: iced::widget::text_input::State::new(),
            export_modal_export_lbl_text_input_content: String::new(),
        }
    }

    fn builder_view(&mut self) -> iced::Element<<Self as iced_native::Program>::Message> {
        let default_padding = 10;

        let board = Board::new(
            &self.level,
            &self.background_image,
            &self.iced_block_map,
            &mut self.board_state,
        )
        .grid(self.grid)
        .active_block(self.active_block.as_ref());

        let helper_bar = iced::Row::new()
            .push(
                iced::Container::new(
                    iced::Checkbox::new(self.grid, "Grid", Message::SetGrid)
                        .size(30)
                        .text_size(30),
                )
                .height(Length::Fill)
                .center_y(),
            )
            .push(
                iced::Container::new(
                    iced::Checkbox::new(self.level.is_dark(), "Dark", Message::SetDark)
                        .size(30)
                        .text_size(30),
                )
                .height(Length::Fill)
                .center_y(),
            )
            .push(
                iced::Button::new(
                    &mut self.export_button_state,
                    iced::Text::new("Export").size(30),
                )
                .padding(default_padding)
                .style(DarkTheme::primary())
                .on_press(Message::OpenExportModal),
            )
            .push(
                iced::Button::new(
                    &mut self.import_button_state,
                    iced::Text::new("Import").size(30),
                )
                .padding(default_padding)
                .style(DarkTheme::primary())
                .on_press(Message::RequestLevelImport),
            )
            .spacing(default_padding)
            .width(Length::Fill);

        let tool_bar = ToolBar::new(
            &self.iced_block_map,
            &mut self.tool_bar_state,
            &self.trash_bin_image,
        );

        let main_content = iced::Row::new()
            .push(
                iced::Column::new()
                    .push(
                        iced::Container::new(board)
                            .padding(default_padding)
                            .style(DarkTheme::primary())
                            .center_x()
                            .center_y()
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .push(
                        iced::Container::new(helper_bar)
                            .width(Length::Fill)
                            .height(Length::Units(70))
                            .style(DarkTheme::primary())
                            .center_y()
                            .padding(default_padding),
                    )
                    .spacing(default_padding)
                    .width(Length::FillPortion(4)),
            )
            .push(iced_native::Container::new(tool_bar).style(DarkTheme::primary()))
            .spacing(default_padding)
            .padding(default_padding);

        iced::Container::new(
            iced::Column::new()
                .push(
                    iced::Container::new(
                        iced::Container::new(
                            iced::Row::new()
                                .push(
                                    iced::Text::new("SS")
                                        .size(40)
                                        .horizontal_alignment(
                                            iced_core::HorizontalAlignment::Center,
                                        )
                                        .vertical_alignment(iced_core::VerticalAlignment::Center),
                                )
                                .spacing(default_padding),
                        )
                        .padding(default_padding),
                    )
                    .height(Length::Units(50))
                    .width(Length::Fill)
                    .style(DarkTheme::secondary()),
                )
                .push(main_content),
        )
        .into()
    }

    fn export_modal_view(&mut self) -> iced::Element<Message> {
        let default_padding = 10;

        let title = iced::Text::new("Export")
            .size(70)
            .horizontal_alignment(iced_core::HorizontalAlignment::Center);

        let content = match self.export_modal_state {
            ExportModalState::ChooseFormat => iced::Column::new()
                .push(iced::Text::new("Export Formats").size(30))
                .push(
                    iced::Button::new(
                        &mut self.export_modal_export_type_lbl_button_state,
                        iced::Text::new("Level File")
                            .horizontal_alignment(iced::HorizontalAlignment::Center)
                            .size(30),
                    )
                    .style(DarkTheme::primary())
                    .on_press(Message::ChangeExportModalState(ExportModalState::Lbl)),
                )
                .align_items(iced::Align::Center)
                .spacing(default_padding),
            ExportModalState::Lbl => iced::Column::new()
                .push(iced::Text::new("Level File").size(30))
                .push(
                    iced::Row::new()
                        .push(iced::Text::new("Filename").size(30))
                        .push(
                            iced::TextInput::new(
                                &mut self.export_modal_export_lbl_text_input_state,
                                "",
                                &self.export_modal_export_lbl_text_input_content,
                                Message::ExportModalLblTextInputChanged,
                            )
                            .padding(default_padding),
                        )
                        .spacing(default_padding),
                )
                .push(
                    iced::Button::new(
                        &mut self.export_modal_export_type_lbl_button_state,
                        iced::Text::new("Export").size(30),
                    )
                    .style(DarkTheme::primary())
                    .on_press(Message::RequestSaveLblExport),
                )
                .align_items(iced::Align::Center)
                .spacing(default_padding),
        };

        let ok_button = iced::Button::new(
            &mut self.export_modal_close_button_state,
            iced::Text::new("Close").size(30),
        )
        .padding(default_padding)
        .style(DarkTheme::primary())
        .on_press(Message::CloseExportModal);

        let main_content = iced_native::Container::new(
            iced_native::Column::new()
                .push(title)
                .push(iced::Space::new(Length::Fill, Length::Fill))
                .push(content)
                .push(iced::Space::new(Length::Fill, Length::Fill))
                .push(ok_button)
                .align_items(iced_core::Align::Center)
                .spacing(default_padding)
                .width(Length::Fill),
        )
        .padding(default_padding)
        .style(DarkTheme::primary())
        .center_x()
        .width(Length::Fill)
        .height(Length::Fill);

        iced::Container::new(main_content)
            .padding(default_padding)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}

impl iced_native::Program for UiApp {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> iced_native::Command<Message> {
        match message {
            Message::AddBlock { index, block } => {
                assert!(
                    self.level.add_block(index, block).is_none(),
                    "index = {}",
                    index
                );
            }
            Message::ImportLevel(level_string) => {
                if let Err(e) = self.level.import_str(&level_string) {
                    return iced::Command::perform(
                        async move { Arc::new(AppError::SksDecode(e)) },
                        |e| Message::ErrorModalMessage(ErrorModalMessage::Open(e)),
                    );
                }
            }
            Message::SetLevelNumber(level_number) => {
                self.level.set_level_number(level_number);
            }
            Message::SetDark(dark) => self.level.set_dark(dark),
            Message::SetGrid(grid) => self.grid = grid,
            Message::ChangeActiveBlock { block } => self.active_block = block,
            Message::CloseExportModal => self.app_state = AppState::Builder,
            Message::OpenExportModal => {
                self.export_modal_state = ExportModalState::ChooseFormat;
                self.app_state = AppState::ExportModal;
            }
            Message::ChangeExportModalState(export_modal_state) => {
                self.export_modal_state = export_modal_state
            }
            Message::NoteModalMessage(msg) => {
                match msg {
                    NoteModalMessage::Open => self.app_state = AppState::NoteModal,
                    NoteModalMessage::Submit => {
                        let text = self.note_modal.take_content();
                        self.active_block = Some(sks::Block::Note { text });
                        self.tool_bar_state.select_block(self.active_block.as_ref());

                        self.app_state = AppState::Builder;
                    }
                    NoteModalMessage::Close => self.app_state = AppState::Builder,
                    _ => {}
                }

                self.note_modal.update(msg)
            }
            Message::ErrorModalMessage(msg) => {
                match &msg {
                    ErrorModalMessage::Open(_) => self.app_state = AppState::ErrorModal,
                    ErrorModalMessage::Close => self.app_state = AppState::Builder,
                }

                self.error_modal.update(msg);
            }
            Message::ExportModalLblTextInputChanged(content) => {
                self.export_modal_export_lbl_text_input_content = content;
            }
            Message::RequestLevelImport => {
                return iced::Command::perform(
                    async {
                        let level_string: Result<_, AppError> = tokio::task::spawn_blocking(|| {
                            let file_path = win_nfd::nfd_open_builder()
                                .default_path(".".as_ref())
                                .execute()?;

                            Ok(std::fs::read_to_string(&file_path)?)
                        })
                        .await
                        .map_err(From::from);

                        level_string
                    },
                    |level_string| match level_string {
                        Ok(Ok(data)) => Message::ImportLevel(data),
                        Err(e) | Ok(Err(e)) => {
                            Message::ErrorModalMessage(ErrorModalMessage::Open(Arc::new(e)))
                        }
                    },
                );
            }
            Message::RequestSaveLblExport => {
                let lbl = self.level.export_str(sks::format::FileFormat::Lbl);

                let mut filename = String::new();
                if !self.export_modal_export_lbl_text_input_content.is_empty() {
                    filename.push_str(&self.export_modal_export_lbl_text_input_content);
                    filename.push_str(".txt");
                }

                return iced::Command::perform(
                    async move {
                        let res = tokio::task::spawn_blocking(move || {
                            let lbl = lbl.map_err(AppError::from)?;

                            let mut nfd = win_nfd::nfd_save_builder();
                            nfd.default_path(".".as_ref())
                                .filetype("level file".as_ref(), "*.txt".as_ref());

                            if !filename.is_empty() {
                                nfd.filename(filename.as_ref());
                            } else {
                                nfd.filename("level.txt".as_ref());
                            }

                            let save_path = nfd.execute()?;

                            std::fs::write(save_path, lbl)?;

                            Result::<_, AppError>::Ok(())
                        })
                        .await
                        .map_err(AppError::from);

                        match res {
                            Ok(Ok(())) => Message::CloseExportModal,
                            Ok(Err(e)) | Err(e) => {
                                Message::ErrorModalMessage(ErrorModalMessage::Open(Arc::new(e)))
                            }
                        }
                    },
                    std::convert::identity,
                );
            }
            Message::Nop => {}
        }

        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        match self.app_state {
            AppState::Builder => self.builder_view(),
            AppState::NoteModal => self.note_modal.view().map(Message::NoteModalMessage),
            AppState::ErrorModal => self.error_modal.view().map(Message::ErrorModalMessage),
            AppState::ExportModal => self.export_modal_view(),
        }
    }
}
