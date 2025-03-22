use std::path::Path;

use async_process::Command;
use iced::{
    widget::{button, column, row, text, text_input},
    window::Settings as WindowSettings,
    Element, Settings, Size, Task,
};

fn main() -> iced::Result {
    iced::application("YT Downloader", YtDownloader::update, YtDownloader::view)
        .window(WindowSettings {
            size: Size::new(800.0, 200.0),
            ..Default::default()
        })
        .centered()
        .run_with(YtDownloader::new)
}

struct YtDownloader {
    url: String,
    save_path: String,
    downloading: bool,
    logs: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputUrl(String),
    InputSavePath(String),
    SaveFileDialog,
    DownloadStart,
    DownloadFirstPartDoneStartingSecond(Result<String, ()>),
    DownloadEnd(Result<(), ()>),
}

impl YtDownloader {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                url: "".to_string(),
                save_path: "".to_string(),
                downloading: false,
                logs: "".to_string(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputUrl(new) => self.url = new,
            Message::InputSavePath(new) => self.save_path = new,
            Message::SaveFileDialog => {
                let file = rfd::FileDialog::new()
                    .set_file_name("video.mp4")
                    .add_filter("Video", &["mp4"])
                    .set_title("Save youtube video")
                    .save_file();

                if let Some(file) = file {
                    self.save_path = file.to_str().unwrap().to_owned();
                }
            }
            Message::DownloadStart => {
                self.downloading = true;

                println!("running yt-dlp...");
                let save_path = self.save_path.clone();
                let url = self.url.clone();
                return Task::perform(
                    async move {
                        let out = Command::new("yt-dlp")
                            .args([
                                "--merge-output-format",
                                "mp4",
                                "-o",
                                &format!("{}", &save_path),
                                &url,
                            ])
                            .status()
                            .await
                            .expect("failed to execute yt-dlp");

                        if !out.success() {
                            return Err(());
                        }

                        Ok(save_path)
                    },
                    Message::DownloadFirstPartDoneStartingSecond,
                );
            }
            Message::DownloadFirstPartDoneStartingSecond(res) => match res {
                Err(()) => {
                    println!("yt-dlp returned error");
                    self.downloading = false;
                }
                Ok(save_path) => {
                    println!("running ffmpeg...");
                    return Task::perform(
                        async move {
                            let with_mov = Path::new(&save_path).with_extension("mov");

                            let out = Command::new("ffmpeg")
                                .args(["-i", &save_path, "-f", "mov", &with_mov.to_str().unwrap()])
                                .status()
                                .await
                                .expect("failed to execute ffmpeg");

                            if !out.success() {
                                eprintln!("ffmpeg return error");
                            }

                            Ok(())
                        },
                        Message::DownloadEnd,
                    );
                }
            },
            Message::DownloadEnd(res) => {
                match res {
                    Err(()) => println!("ffmpeg returned error"),
                    Ok(()) => println!("download & conversion successful!"),
                }
                self.downloading = false;
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let input_url = row([
            text("YouTube URL:").into(),
            text_input("https://www.youtube.com/watch?v=dQw4w9WgXcQ", &self.url)
                .on_input(Message::InputUrl)
                .into(),
        ]
        .into_iter());

        let input_save_path = row([
            text("Save at:").into(),
            text_input("", &self.save_path)
                .on_input(Message::InputSavePath)
                .into(),
            button("Select path...")
                .on_press(Message::SaveFileDialog)
                .into(),
        ]
        .into_iter());

        let button_download = button("Download")
            .on_press_maybe((!self.downloading).then_some(Message::DownloadStart));

        column![input_url, input_save_path, button_download].into()
    }
}
