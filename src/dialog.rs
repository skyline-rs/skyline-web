use crate::{Background, BootDisplay, Webpage};
use ramhorns::{Template, Content};

#[derive(Content)]
pub struct Dialog {
    #[md]
    text: String,
    left_button: String,
    right_button: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DialogOption {
    Left,
    Right
}

impl Dialog {
    pub fn new<S1, S2, S3>(text: S1, left_button: S2, right_button: S3) -> Self
        where S1: Into<String>,
              S2: Into<String>,
              S3: Into<String>,
    {
        Self {
            text: text.into(),
            left_button: left_button.into(),
            right_button: right_button.into()
        }
    }

    pub fn yes_no<S: Into<String>>(message: S) -> bool {
        match Dialog::new(message, "Yes", "No").show() {
            DialogOption::Left => true,
            DialogOption::Right => false
        }
    }
    
    pub fn ok_cancel<S: Into<String>>(message: S) -> bool {
        match Dialog::new(message, "Ok", "Cancel").show() {
            DialogOption::Left => true,
            DialogOption::Right => false
        }
    }

    pub fn show(&self) -> DialogOption {
        let tpl = Template::new(include_str!("templates/dialog.html")).unwrap();

        let response = Webpage::new()
            .background(Background::BlurredScreenshot)
            .file("index.html", &tpl.render(self))
            .boot_display(BootDisplay::BlurredScreenshot)
            .open()
            .unwrap();

        match response.get_last_url().unwrap() {
            "http://localhost/left" => DialogOption::Left,
            "http://localhost/right" => DialogOption::Right,
            // Until this is reworked to offer a default option on forceful closure
            _ => DialogOption::Right
        } 
    }
}
