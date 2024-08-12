use std::error::Error;

use miette::Diagnostic;
use poem_openapi::payload::Json;

#[derive(poem_openapi::Object)]
pub struct PoemResponse {
    code: String,
    message: String,
    detail: String,
    help: Option<String>,
    doc: Option<String>,
}

pub trait HaoyongError {
    fn to_brief_report(&self) -> String;
    fn to_detailed_singleline_report(&self) -> String;
    fn to_detailed_multiline_report(&self) -> String;
    fn into_fancy_cli_report(self) -> String;
    fn into_poem_response(self) -> PoemResponse;
}

impl<T> HaoyongError for T
where
    T: Error + Diagnostic + Send + Sync + 'static,
{
    fn to_brief_report(&self) -> String {
        self.to_string()
    }

    fn to_detailed_singleline_report(&self) -> String {
        let shallow = self.to_string();
        let chain = std::iter::successors(self.source(), |&e| e.source())
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join(" <<< ");
        format!("{shallow} | {chain}")
    }

    fn to_detailed_multiline_report(&self) -> String {
        let shallow = self.to_string();
        let chain = std::iter::successors(self.source(), |&e| e.source())
            .zip(1..)
            .map(|(err, depth)| format!("    [{depth}] {}", err.to_string()))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{shallow}\n\nCaused By:\n{chain}")
    }

    fn into_fancy_cli_report(self) -> String {
        format!("{:?}", miette::Report::from(self))
    }

    fn into_poem_response(self) -> PoemResponse {
        PoemResponse {
            code: self.code().map(|code| code.to_string()).unwrap(), // TODO
            message: self.to_brief_report(),
            detail: self.to_detailed_singleline_report(),
            help: self.help().map(|help| help.to_string()),
            doc: self.url().map(|url| url.to_string()),
        }
    }
}

pub type HaoyongPoemResponse = Json<PoemResponse>;

pub trait IntoPoemResult<T, E> {
    fn map_err_to_poem_response(self) -> Result<T, HaoyongPoemResponse>;
}

impl<T, E> IntoPoemResult<T, E> for Result<T, E>
where
    E: HaoyongError,
{
    fn map_err_to_poem_response(self) -> Result<T, HaoyongPoemResponse> {
        self.map_err(|err| err.into_poem_response()).map_err(Json)
    }
}
