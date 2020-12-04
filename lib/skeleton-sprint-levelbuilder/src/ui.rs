mod style;
mod widgets;

use self::{
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
pub enum Message {
    AddBlock { index: usize, block: sks::Block },
    ImportLevel(String),
    SetLevelNumber(Option<LevelNumber>),
    SetDark(bool),
    SetGrid(bool),
    ChangeActiveBlock { block: Option<sks::Block> },
    OpenNoteModal,
    OpenErrorModal(Arc<AppError>),

    NoteModalInputChanged(String),
    NoteModalSubmit { is_success: bool },

    CloseErrorModal,

    RequestLevelImport,

    Nop,
}

pub struct UiApp {
    pub level: sks::Level,
    active_block: Option<sks::Block>,

    grid: bool,

    iced_block_map: crate::IcedBlockMap,
    iced_background_image: iced_native::image::Handle,
    iced_trash_bin_image: iced_native::image::Handle,

    board_state: widgets::board::State,
    tool_bar_state: widgets::tool_bar::State,
    import_button_state: iced::button::State,
    export_button_state: iced::button::State,

    app_state: AppState,

    note_modal_close_button_state: iced::widget::button::State,
    note_modal_text_input_state: iced::widget::text_input::State,
    note_modal_text_input_content: String,

    error_modal_error: Option<Arc<AppError>>,
    error_modal_ok_button_state: iced::widget::button::State,
}

impl UiApp {
    pub fn new(
        iced_block_map: crate::IcedBlockMap,
        iced_background_image: iced_native::image::Handle,
        iced_trash_bin_image: iced_native::image::Handle,
    ) -> Self {
        Self {
            level: sks::Level::new(),
            active_block: None,

            grid: true,

            iced_block_map,
            iced_background_image,
            iced_trash_bin_image,

            board_state: widgets::board::State::new(),
            tool_bar_state: widgets::tool_bar::State::new(),
            import_button_state: iced::button::State::new(),
            export_button_state: iced::button::State::new(),

            app_state: AppState::Builder,

            note_modal_close_button_state: iced::widget::button::State::new(),
            note_modal_text_input_state: iced::widget::text_input::State::new(),
            note_modal_text_input_content: String::new(),

            error_modal_error: None,
            error_modal_ok_button_state: iced::widget::button::State::new(),
        }
    }

    fn builder_view(
        &mut self,
    ) -> iced_native::Element<
        <Self as iced_native::Program>::Message,
        <Self as iced_native::Program>::Renderer,
    > {
        let default_padding = 10;

        let board = Board::new(
            &self.level,
            &self.iced_background_image,
            &self.iced_block_map,
            &mut self.board_state,
        )
        .grid(self.grid)
        .active_block(self.active_block.as_ref());

        let tool_bar = ToolBar::new(
            &self.iced_block_map,
            &mut self.tool_bar_state,
            &self.iced_trash_bin_image,
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
                        iced::Container::new(
                            iced::Row::new()
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
                                        iced::Checkbox::new(
                                            self.level.is_dark(),
                                            "Dark",
                                            Message::SetDark,
                                        )
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
                                    .on_press(Message::Nop),
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
                                .width(Length::Fill),
                        )
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

    fn note_modal_view(
        &mut self,
    ) -> iced_native::Element<
        <Self as iced_native::Program>::Message,
        <Self as iced_native::Program>::Renderer,
    > {
        let default_padding = 10;
        
        let title = iced::Text::new("Note Content")
            .size(70)
            .horizontal_alignment(iced_core::HorizontalAlignment::Center);

        let input = iced_native::TextInput::new(
            &mut self.note_modal_text_input_state,
            "note content...",
            &self.note_modal_text_input_content,
            Message::NoteModalInputChanged,
        )
        .on_submit(Message::NoteModalSubmit { is_success: true })
        .size(50)
        .padding(default_padding);

        let exit_button = iced_native::Button::new(
            &mut self.note_modal_close_button_state,
            iced::Text::new("Exit").size(30),
        )
        .padding(default_padding)
        .style(DarkTheme::primary())
        .on_press(Message::NoteModalSubmit { is_success: false });

        let main_content = iced_native::Container::new(
            iced_native::Column::new()
                .push(title)
                .push(input)
                .push(iced_native::Space::new(Length::Fill, Length::Fill))
                .push(exit_button)
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

    fn error_modal_view(
        &mut self,
    ) -> iced_native::Element<
        <Self as iced_native::Program>::Message,
        <Self as iced_native::Program>::Renderer,
    > {
        let default_padding = 10;
        
        let title = iced::Text::new("Error")
            .size(70)
            .horizontal_alignment(iced_core::HorizontalAlignment::Center);

        let error_msg = iced::Text::new(
            self.error_modal_error
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Error was not specified".into()),
        )
        .size(70)
        .horizontal_alignment(iced_core::HorizontalAlignment::Center);

        let ok_button = iced::Button::new(
            &mut self.error_modal_ok_button_state,
            iced::Text::new("Ok").size(30),
        )
        .padding(default_padding)
        .style(DarkTheme::primary())
        .on_press(Message::NoteModalSubmit { is_success: false });

        let main_content = iced_native::Container::new(
            iced_native::Column::new()
                .push(title)
                .push(iced::Space::new(Length::Fill, Length::Fill))
                .push(error_msg)
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
                        async move { Message::OpenErrorModal(Arc::new(AppError::SksDecode(e))) },
                        std::convert::identity,
                    );
                }
            }
            Message::SetLevelNumber(level_number) => {
                self.level.set_level_number(level_number);
            }
            Message::SetDark(dark) => {
                self.level.set_dark(dark);
            }
            Message::SetGrid(grid) => {
                self.grid = grid;
            }
            Message::ChangeActiveBlock { block } => {
                self.active_block = block;
            }
            Message::OpenNoteModal => {
                self.note_modal_text_input_state = iced_native::widget::text_input::State::new();
                self.note_modal_text_input_content.clear();
                self.app_state = AppState::NoteModal;
            }
            Message::OpenErrorModal(e) => {
                self.error_modal_error = Some(e);
                self.app_state = AppState::ErrorModal;
            }
            Message::CloseErrorModal => {
                self.app_state = AppState::Builder;
            }
            Message::NoteModalInputChanged(content) => {
                self.note_modal_text_input_content = content;
            }
            Message::NoteModalSubmit { is_success } => {
                if is_success {
                    let text = std::mem::take(&mut self.note_modal_text_input_content);
                    self.active_block = Some(sks::Block::Note { text });
                    self.tool_bar_state.select_block(self.active_block.as_ref());
                }

                self.app_state = AppState::Builder;
            }
            Message::RequestLevelImport => {
                return iced::Command::perform(
                    async {
                        let level: Result<_, AppError> = tokio::task::spawn_blocking(|| {
                            let file_path = win_nfd::nfd_open_builder()
                                .default_path(".".as_ref())
                                .execute()?;

                            let file_str = std::fs::read_to_string(&file_path)?;

                            Ok(file_str)
                        })
                        .await
                        .unwrap();

                        let level = level.expect("SKS LEVEL");

                        Message::ImportLevel(level)
                    },
                    std::convert::identity,
                );
            }
            Message::Nop => {}
        }

        iced_native::Command::none()
    }

    fn view(&mut self) -> iced_native::Element<Self::Message, Self::Renderer> {
        match self.app_state {
            AppState::Builder => self.builder_view(),
            AppState::NoteModal => self.note_modal_view(),
            AppState::ErrorModal => self.error_modal_view(),
        }
    }
}
