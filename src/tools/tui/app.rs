use std::io::stdout;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Error, Result, anyhow};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    prelude::{Margin, Rect},
};
use tokio::runtime::Handle;
use tokio::task;

use super::ui;
use crate::engine::EngineIdentity;
use crate::tools::sol;
use rust_decimal::Decimal;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct InteractiveContext {
    pub rpc_endpoint: String,
    pub dry_run: bool,
    pub wallets: Vec<WalletSummary>,
}

#[derive(Clone)]
pub struct ExecutionResources {
    pub rpc: Arc<RpcClient>,
    pub identity: EngineIdentity,
    pub compute_unit_lamports: u64,
}

#[derive(Clone, Debug)]
pub struct WalletSummary {
    pub title: String,
    pub subtitle: Option<String>,
    pub is_primary: bool,
}

impl WalletSummary {
    pub fn primary(title: String, subtitle: Option<String>) -> Self {
        Self {
            title,
            subtitle,
            is_primary: true,
        }
    }

    pub fn secondary(title: String, subtitle: Option<String>) -> Self {
        Self {
            title,
            subtitle,
            is_primary: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolsAction {
    Transfer,
    WrapSol,
    PartialUnwrap,
}

impl ToolsAction {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Transfer => "Transfer SOL",
            Self::WrapSol => "Wrap SOL",
            Self::PartialUnwrap => "Partial Unwrap",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Transfer => "发送原生 SOL 到目标地址，适合日常转账或风控调账。",
            Self::WrapSol => "将指定数量的 SOL 包装为 WSOL 并同步到 ATA。",
            Self::PartialUnwrap => "从 WSOL 账户中提取一部分 SOL，自动关闭临时账户。",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusArea {
    Wallets,
    Actions,
}

impl FocusArea {
    fn next(self) -> Self {
        match self {
            Self::Wallets => Self::Actions,
            Self::Actions => Self::Wallets,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Wallets => "Wallets",
            Self::Actions => "Actions",
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct LayoutState {
    pub wallet_area: Option<Rect>,
    pub wallet_inner: Option<Rect>,
    pub action_area: Option<Rect>,
    pub action_inner: Option<Rect>,
}

pub struct ToolsApp {
    pub context: InteractiveContext,
    pub wallets: Vec<WalletSummary>,
    pub wallet_index: usize,
    pub actions: Vec<ToolsAction>,
    pub action_index: usize,
    pub focus: FocusArea,
    pub logs: Vec<String>,
    pub should_quit: bool,
    pub layout: LayoutState,
    pub view: ViewMode,
    pub form: Option<ActionForm>,
    pub pending: Option<PendingAction>,
    executor: ExecutionResources,
    runtime: Handle,
}

impl ToolsApp {
    fn new(context: InteractiveContext, executor: ExecutionResources, runtime: Handle) -> Self {
        let mut wallets = context.wallets.clone();
        if wallets.is_empty() {
            wallets.push(WalletSummary::primary(
                "Primary Wallet".to_string(),
                Some("未加载主钱包".to_string()),
            ));
        }

        let mut logs = Vec::with_capacity(32);
        logs.push(format!("已连接 RPC：{}", context.rpc_endpoint));
        if context.dry_run {
            logs.push("dry-run 已启用，所有操作都会发送到沙盒 RPC".to_string());
        } else {
            logs.push("当前环境：主网（实际操作请仔细确认）".to_string());
        }

        Self {
            context,
            wallets,
            wallet_index: 0,
            actions: vec![
                ToolsAction::Transfer,
                ToolsAction::WrapSol,
                ToolsAction::PartialUnwrap,
            ],
            action_index: 0,
            focus: FocusArea::Wallets,
            logs,
            should_quit: false,
            layout: LayoutState::default(),
            view: ViewMode::Dashboard,
            form: None,
            pending: None,
            executor,
            runtime,
        }
    }

    fn current_action(&self) -> Option<ToolsAction> {
        self.actions.get(self.action_index).copied()
    }

    fn current_wallet(&self) -> Option<&WalletSummary> {
        self.wallets.get(self.wallet_index)
    }

    fn move_selection(&mut self, delta: i32) {
        match self.focus {
            FocusArea::Wallets => {
                if self.wallets.is_empty() {
                    return;
                }
                let len = self.wallets.len() as i32;
                let mut idx = self.wallet_index as i32 + delta;
                if idx < 0 {
                    idx = 0;
                } else if idx >= len {
                    idx = len - 1;
                }
                self.wallet_index = idx as usize;
            }
            FocusArea::Actions => {
                if self.actions.is_empty() {
                    return;
                }
                let len = self.actions.len() as i32;
                let mut idx = self.action_index as i32 + delta;
                if idx < 0 {
                    idx = 0;
                } else if idx >= len {
                    idx = len - 1;
                }
                self.action_index = idx as usize;
            }
        }
    }

    fn push_log<S: Into<String>>(&mut self, message: S) {
        self.logs.push(message.into());
        if self.logs.len() > 200 {
            self.logs.drain(..self.logs.len() - 200);
        }
    }

    fn handle_activation(&mut self) {
        if self.view != ViewMode::Dashboard {
            return;
        }
        if let Some(action) = self.current_action() {
            self.open_form(action);
        }
    }

    fn handle_mouse_click(&mut self, column: u16, row: u16) {
        if let Some(inner) = self.layout.wallet_inner {
            if point_in_rect(inner, column, row) {
                let idx = row.saturating_sub(inner.y) as usize;
                if idx < self.wallets.len() {
                    self.wallet_index = idx;
                    self.focus = FocusArea::Wallets;
                }
                return;
            }
        }

        if let Some(inner) = self.layout.action_inner {
            if point_in_rect(inner, column, row) {
                let idx = row.saturating_sub(inner.y) as usize;
                if idx < self.actions.len() {
                    self.action_index = idx;
                    self.focus = FocusArea::Actions;
                }
            }
        }
    }

    fn handle_scroll(&mut self, direction: i32) {
        let delta = if direction > 0 { 1 } else { -1 };
        self.move_selection(delta);
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }
        match self.view {
            ViewMode::Dashboard => self.handle_dashboard_key(key),
            ViewMode::Form => self.handle_form_key(key),
            ViewMode::Confirm => self.handle_confirm_key(key),
            ViewMode::Executing => {}
        }
    }

    fn on_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(mouse.column, mouse.row);
            }
            MouseEventKind::ScrollDown => self.handle_scroll(1),
            MouseEventKind::ScrollUp => self.handle_scroll(-1),
            _ => {}
        }
    }

    pub fn focus_label(&self) -> &'static str {
        self.focus.label()
    }

    pub fn action_details(&self) -> String {
        match self.view {
            ViewMode::Dashboard => {
                let mut lines = Vec::new();
                if let Some(action) = self.current_action() {
                    lines.push(format!("操作：{}", action.title()));
                    lines.push(action.description().to_string());
                }
                if let Some(wallet) = self.current_wallet() {
                    lines.push(String::new());
                    lines.push(format!("当前钱包：{}", wallet.title));
                    if let Some(sub) = &wallet.subtitle {
                        lines.push(sub.clone());
                    }
                }
                lines.push(String::new());
                lines.push("Enter：进入操作菜单 | Tab：切换区域 | q：退出".to_string());
                lines.join("\n")
            }
            ViewMode::Form => {
                if let Some(form) = &self.form {
                    form.describe()
                } else {
                    "表单初始化中...".to_string()
                }
            }
            ViewMode::Confirm => {
                if let Some(pending) = &self.pending {
                    pending.describe_confirmation()
                } else {
                    String::new()
                }
            }
            ViewMode::Executing => "执行中，请稍候...".to_string(),
        }
    }

    pub fn visible_logs(&self, max_lines: usize) -> Vec<String> {
        if self.logs.is_empty() || max_lines == 0 {
            return Vec::new();
        }
        let len = self.logs.len();
        let start = len.saturating_sub(max_lines);
        self.logs[start..].to_vec()
    }

    fn handle_dashboard_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab | KeyCode::BackTab => self.focus = self.focus.next(),
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Enter => self.handle_activation(),
            _ => {}
        }
    }

    fn handle_form_key(&mut self, key: KeyEvent) {
        if let Some(form) = self.form.as_mut() {
            if form.handle_key(key) {
                return;
            }
        }
        match key.code {
            KeyCode::Esc => self.cancel_form(),
            KeyCode::Enter => self.attempt_form_submission(),
            KeyCode::Tab | KeyCode::BackTab => {
                if let Some(form) = self.form.as_mut() {
                    form.cycle_focus();
                }
            }
            KeyCode::Up => {
                if let Some(form) = self.form.as_mut() {
                    form.move_focus(-1);
                }
            }
            KeyCode::Down => {
                if let Some(form) = self.form.as_mut() {
                    form.move_focus(1);
                }
            }
            _ => {}
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.execute_pending_action(),
            KeyCode::Esc => self.return_to_form(),
            KeyCode::Char('b') => self.return_to_form(),
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn open_form(&mut self, action: ToolsAction) {
        let wallet = self
            .current_wallet()
            .map(|w| w.title.clone())
            .unwrap_or_else(|| "未知钱包".to_string());
        let form = ActionForm::new(action, wallet);
        self.form = Some(form);
        self.view = ViewMode::Form;
    }

    fn cancel_form(&mut self) {
        self.form = None;
        self.pending = None;
        self.view = ViewMode::Dashboard;
    }

    fn attempt_form_submission(&mut self) {
        if let Some(form) = self.form.as_mut() {
            match form.validate() {
                Ok(pending) => {
                    self.pending = Some(pending);
                    self.view = ViewMode::Confirm;
                }
                Err(err) => {
                    form.message = Some(err);
                }
            }
        }
    }

    fn return_to_form(&mut self) {
        if self.form.is_some() {
            self.view = ViewMode::Form;
        } else {
            self.view = ViewMode::Dashboard;
        }
    }

    fn execute_pending_action(&mut self) {
        if self.view != ViewMode::Confirm {
            return;
        }
        let pending = match self.pending.clone() {
            Some(value) => value,
            None => return,
        };
        self.view = ViewMode::Executing;
        let executor = self.executor.clone();
        let mut logs = Vec::new();
        let result: Result<String, Error> = self.runtime.block_on(async move {
            match pending.params {
                ActionParams::Transfer { recipient, amount } => {
                    let recipient_pubkey = Pubkey::from_str(&recipient)
                        .map_err(|err| anyhow!("接收者地址格式错误: {err}"))?;
                    let lamports = sol::decimal_sol_to_lamports(&amount)?;
                    let signature = sol::transfer_sol(
                        &executor.rpc,
                        &executor.identity,
                        &recipient_pubkey,
                        lamports,
                        executor.compute_unit_lamports,
                    )
                    .await?;
                    Ok(format!("Transfer 成功：{}", signature))
                }
                ActionParams::Wrap { amount } => {
                    let lamports = sol::decimal_sol_to_lamports(&amount)?;
                    let signature = sol::wrap_sol(
                        &executor.rpc,
                        &executor.identity,
                        lamports,
                        executor.compute_unit_lamports,
                    )
                    .await?;
                    Ok(format!("Wrap 成功：{}", signature))
                }
                ActionParams::Unwrap { amount } => {
                    let lamports = sol::decimal_sol_to_lamports(&amount)?;
                    let signature = sol::partial_unwrap_wsol(
                        &executor.rpc,
                        &executor.identity,
                        lamports,
                        executor.compute_unit_lamports,
                    )
                    .await?;
                    Ok(format!("Partial Unwrap 成功：{}", signature))
                }
            }
        });

        match result {
            Ok(message) => {
                logs.push(message);
            }
            Err(err) => {
                logs.push(format!("操作失败：{err}"));
                if let Some(form) = self.form.as_mut() {
                    form.message = Some(err.to_string());
                }
            }
        }
        for log in logs {
            self.push_log(log);
        }
        self.view = ViewMode::Dashboard;
        self.pending = None;
        self.form = None;
    }
}

pub async fn run_interactive_tui(
    context: InteractiveContext,
    executor: ExecutionResources,
) -> Result<()> {
    let handle = Handle::current();
    task::spawn_blocking(move || run_blocking_app(context, executor, handle)).await?
}

fn run_blocking_app(
    context: InteractiveContext,
    executor: ExecutionResources,
    handle: Handle,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let res = run_app_loop(&mut terminal, context, executor, handle);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    res
}

fn run_app_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    context: InteractiveContext,
    executor: ExecutionResources,
    runtime: Handle,
) -> Result<()> {
    let tick_rate = Duration::from_millis(200);
    let mut app = ToolsApp::new(context, executor, runtime);

    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;
        if app.should_quit {
            break;
        }
        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => app.on_key(key),
                Event::Mouse(mouse) => app.on_mouse(mouse),
                Event::Resize(_, _) => {}
                Event::FocusGained | Event::FocusLost => {}
                Event::Paste(_) => {}
            }
        }
    }
    Ok(())
}

fn point_in_rect(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

impl LayoutState {
    pub fn update_wallet_area(&mut self, area: Rect) {
        self.wallet_area = Some(area);
        self.wallet_inner = Some(area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        }));
    }

    pub fn update_action_area(&mut self, area: Rect) {
        self.action_area = Some(area);
        self.action_inner = Some(area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        }));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Dashboard,
    Form,
    Confirm,
    Executing,
}

#[derive(Clone)]
pub struct ActionForm {
    pub action: ToolsAction,
    pub wallet_label: String,
    pub fields: Vec<InputField>,
    pub focus_index: usize,
    pub message: Option<String>,
}

impl ActionForm {
    fn new(action: ToolsAction, wallet_label: String) -> Self {
        let fields = match action {
            ToolsAction::Transfer => vec![
                InputField::new("接收地址", "输入 Base58 地址", InputKind::Address),
                InputField::new("金额 (SOL)", "例如 0.25", InputKind::Amount),
            ],
            ToolsAction::WrapSol => vec![InputField::new(
                "金额 (SOL)",
                "转换为 WSOL 的数量",
                InputKind::Amount,
            )],
            ToolsAction::PartialUnwrap => vec![InputField::new(
                "金额 (SOL)",
                "解包的 SOL 数量",
                InputKind::Amount,
            )],
        };
        Self {
            action,
            wallet_label,
            fields,
            focus_index: 0,
            message: None,
        }
    }

    fn current_field_mut(&mut self) -> Option<&mut InputField> {
        self.fields.get_mut(self.focus_index)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if let Some(field) = self.current_field_mut() {
            if field.handle_key(key) {
                return true;
            }
        }
        false
    }

    fn cycle_focus(&mut self) {
        if self.fields.is_empty() {
            return;
        }
        self.focus_index = (self.focus_index + 1) % self.fields.len();
    }

    fn move_focus(&mut self, delta: i32) {
        if self.fields.is_empty() {
            return;
        }
        let len = self.fields.len() as i32;
        let mut idx = self.focus_index as i32 + delta;
        if idx < 0 {
            idx = 0;
        } else if idx >= len {
            idx = len - 1;
        }
        self.focus_index = idx as usize;
    }

    fn describe(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("操作：{}", self.action.title()));
        lines.push(format!("钱包：{}", self.wallet_label));
        lines.push(String::new());
        for (idx, field) in self.fields.iter().enumerate() {
            let prefix = if idx == self.focus_index { "▸" } else { " " };
            lines.push(format!(
                "{} {}: {}",
                prefix,
                field.label,
                field.display_value()
            ));
        }
        lines.push(String::new());
        lines.push("Tab/方向键：切换输入 | Enter：提交 | Esc：取消".to_string());
        if let Some(message) = &self.message {
            lines.push(String::new());
            lines.push(format!("提示：{}", message));
        }
        lines.join("\n")
    }

    fn validate(&self) -> Result<PendingAction, String> {
        match self.action {
            ToolsAction::Transfer => {
                if self.fields.len() < 2 {
                    return Err("表单缺少字段".into());
                }
                let address = self.fields[0].value.trim();
                if address.is_empty() {
                    return Err("请输入接收地址".into());
                }
                let amount = parse_decimal(self.fields[1].value.trim())?;
                Ok(PendingAction {
                    action: self.action,
                    params: ActionParams::Transfer {
                        recipient: address.to_string(),
                        amount,
                    },
                })
            }
            ToolsAction::WrapSol => {
                let amount = parse_decimal(self.fields[0].value.trim())?;
                Ok(PendingAction {
                    action: self.action,
                    params: ActionParams::Wrap { amount },
                })
            }
            ToolsAction::PartialUnwrap => {
                let amount = parse_decimal(self.fields[0].value.trim())?;
                Ok(PendingAction {
                    action: self.action,
                    params: ActionParams::Unwrap { amount },
                })
            }
        }
    }
}

fn parse_decimal(input: &str) -> Result<Decimal, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("请输入金额".into());
    }
    Decimal::from_str(trimmed).map_err(|err| format!("金额格式错误: {err}"))
}

#[derive(Clone, Debug)]
pub struct InputField {
    label: &'static str,
    placeholder: &'static str,
    value: String,
    cursor: usize,
    kind: InputKind,
}

impl InputField {
    fn new(label: &'static str, placeholder: &'static str, kind: InputKind) -> Self {
        Self {
            label,
            placeholder,
            value: String::new(),
            cursor: 0,
            kind,
        }
    }

    fn display_value(&self) -> String {
        if self.value.is_empty() {
            format!("({})", self.placeholder)
        } else {
            self.value.clone()
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor < self.value.len() {
                    self.cursor += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor = 0;
                true
            }
            KeyCode::End => {
                self.cursor = self.value.len();
                true
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
                true
            }
            KeyCode::Char(ch) => {
                if self.kind.accepts(ch) {
                    self.value.insert(self.cursor, ch);
                    self.cursor += 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InputKind {
    Address,
    Amount,
}

impl InputKind {
    fn accepts(self, ch: char) -> bool {
        match self {
            InputKind::Address => ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-'),
            InputKind::Amount => ch.is_ascii_digit() || ch == '.',
        }
    }
}

#[derive(Clone, Debug)]
pub struct PendingAction {
    pub action: ToolsAction,
    pub params: ActionParams,
}

impl PendingAction {
    pub fn describe_confirmation(&self) -> String {
        let mut lines = Vec::new();
        lines.push("确认操作".to_string());
        lines.push(format!("类型：{}", self.action.title()));
        match &self.params {
            ActionParams::Transfer { recipient, amount } => {
                lines.push(format!("接收者：{}", recipient));
                lines.push(format!("金额：{} SOL", amount));
            }
            ActionParams::Wrap { amount } => {
                lines.push(format!("Wrap 金额：{} SOL", amount));
            }
            ActionParams::Unwrap { amount } => {
                lines.push(format!("Partial Unwrap 金额：{} SOL", amount));
            }
        }
        lines.push(String::new());
        lines.push("Enter：确认并执行 | Esc：返回表单 | q：退出".to_string());
        lines.join("\n")
    }
}

#[derive(Clone, Debug)]
pub enum ActionParams {
    Transfer { recipient: String, amount: Decimal },
    Wrap { amount: Decimal },
    Unwrap { amount: Decimal },
}
