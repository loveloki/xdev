use comfy_table::{ContentArrangement, Table};

/// 表格主题枚举
#[derive(Debug, Clone, Copy)]
pub enum TableTheme {
    Default,
    Compact,
    Detailed,
}

/// 表格样式配置
#[derive(Debug, Clone)]
pub struct TableStyle {
    pub theme: TableTheme,
    pub max_width: Option<u16>,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            theme: TableTheme::Default,
            max_width: None,
        }
    }
}

/// 创建带样式的表格
pub fn create_styled_table(style: &TableStyle) -> Table {
    let mut table = Table::new();

    // 应用主题样式
    apply_theme(&mut table, style.theme);

    // 设置最大宽度
    if let Some(max_width) = style.max_width {
        table.set_width(max_width);
    }

    table
}

/// 应用主题样式
fn apply_theme(table: &mut Table, theme: TableTheme) {
    match theme {
        TableTheme::Default => {
            table.set_content_arrangement(ContentArrangement::Dynamic);
        }
        TableTheme::Compact => {
            table.set_content_arrangement(ContentArrangement::Dynamic);
        }
        TableTheme::Detailed => {
            table.set_content_arrangement(ContentArrangement::Dynamic);
        }
    }
}

/// 设置表格头部
pub fn set_table_header(table: &mut Table, headers: Vec<String>) {
    table.set_header(headers);
}

/// 添加表格行
pub fn add_table_row(table: &mut Table, row: Vec<String>) {
    table.add_row(row);
}

/// 打印表格
pub fn print_table(table: &Table) {
    println!("{table}");
}

/// 创建配置表格（使用默认样式）
pub fn create_config_table() -> Table {
    create_styled_table(&TableStyle::default())
}

/// 创建订阅列表表格（使用紧凑样式）
pub fn create_subscription_table() -> Table {
    let style = TableStyle {
        theme: TableTheme::Compact,
        ..Default::default()
    };
    create_styled_table(&style)
}

/// 创建备份列表表格（使用详细样式）
pub fn create_backup_table() -> Table {
    let style = TableStyle {
        theme: TableTheme::Detailed,
        ..Default::default()
    };
    create_styled_table(&style)
}
