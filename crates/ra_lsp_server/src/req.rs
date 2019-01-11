use languageserver_types::{Location, Position, Range, TextDocumentIdentifier, Url};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use url_serde;

pub use languageserver_types::{
    notification::*, request::*, ApplyWorkspaceEditParams, CodeActionParams, CompletionParams,
    CompletionResponse, DocumentOnTypeFormattingParams, DocumentSymbolParams,
    DocumentSymbolResponse, ExecuteCommandParams, Hover, InitializeResult,
    PublishDiagnosticsParams, ReferenceParams, SignatureHelp, TextDocumentEdit,
    TextDocumentPositionParams, TextEdit, WorkspaceEdit, WorkspaceSymbolParams,
};

pub enum SyntaxTree {}

impl Request for SyntaxTree {
    type Params = SyntaxTreeParams;
    type Result = String;
    const METHOD: &'static str = "m/syntaxTree";
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyntaxTreeParams {
    pub text_document: TextDocumentIdentifier,
}

pub enum ExtendSelection {}

impl Request for ExtendSelection {
    type Params = ExtendSelectionParams;
    type Result = ExtendSelectionResult;
    const METHOD: &'static str = "m/extendSelection";
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtendSelectionParams {
    pub text_document: TextDocumentIdentifier,
    pub selections: Vec<Range>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtendSelectionResult {
    pub selections: Vec<Range>,
}

pub enum SelectionRangeRequest {}

impl Request for SelectionRangeRequest {
    type Params = TextDocumentPositionParams;
    type Result = Vec<SelectionRange>;
    const METHOD: &'static str = "textDocument/selectionRange";
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectionRange {
    pub range: Range,
}

pub enum FindMatchingBrace {}

impl Request for FindMatchingBrace {
    type Params = FindMatchingBraceParams;
    type Result = Vec<Position>;
    const METHOD: &'static str = "m/findMatchingBrace";
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FindMatchingBraceParams {
    pub text_document: TextDocumentIdentifier,
    pub offsets: Vec<Position>,
}

pub enum DecorationsRequest {}

impl Request for DecorationsRequest {
    type Params = TextDocumentIdentifier;
    type Result = Vec<Decoration>;
    const METHOD: &'static str = "m/decorationsRequest";
}

pub enum PublishDecorations {}

impl Notification for PublishDecorations {
    type Params = PublishDecorationsParams;
    const METHOD: &'static str = "m/publishDecorations";
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishDecorationsParams {
    #[serde(with = "url_serde")]
    pub uri: Url,
    pub decorations: Vec<Decoration>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Decoration {
    pub range: Range,
    pub tag: &'static str,
}

pub enum ParentModule {}

impl Request for ParentModule {
    type Params = TextDocumentPositionParams;
    type Result = Vec<Location>;
    const METHOD: &'static str = "m/parentModule";
}

pub enum JoinLines {}

impl Request for JoinLines {
    type Params = JoinLinesParams;
    type Result = SourceChange;
    const METHOD: &'static str = "m/joinLines";
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JoinLinesParams {
    pub text_document: TextDocumentIdentifier,
    pub range: Range,
}

pub enum OnEnter {}

impl Request for OnEnter {
    type Params = TextDocumentPositionParams;
    type Result = Option<SourceChange>;
    const METHOD: &'static str = "m/onEnter";
}

pub enum Runnables {}

impl Request for Runnables {
    type Params = RunnablesParams;
    type Result = Vec<Runnable>;
    const METHOD: &'static str = "m/runnables";
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunnablesParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Option<Position>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Runnable {
    pub range: Range,
    pub label: String,
    pub bin: String,
    pub args: Vec<String>,
    pub env: FxHashMap<String, String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SourceChange {
    pub label: String,
    pub workspace_edit: WorkspaceEdit,
    pub cursor_position: Option<TextDocumentPositionParams>,
}

pub enum InternalFeedback {}

impl Notification for InternalFeedback {
    const METHOD: &'static str = "internalFeedback";
    type Params = String;
}
