use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub title: Style,
    pub title_border: Style,
    pub header: Style,
    pub ok: Style,
    pub warn: Style,
    pub error: Style,
    pub dim: Style,
    pub strong: Style,
    pub selected: Style,
    pub key: Style,
    pub border_error: Style,
    pub tab_active: Style,
    pub tab_inactive: Style,
    pub gauge: Style,
    pub status_left: Style,
    pub status_fill: Style,
    pub status_right: Style,
    pub danger_bold: Style,
}

impl Theme {
    pub fn from_config(value: &str) -> Self {
        match normalize_theme_name(value).as_str() {
            "system" | "terminal" => terminal(),
            "high_contrast" | "high-contrast" | "contrast" => high_contrast(),
            "no_color" | "no-color" | "mono" | "monochrome" => no_color(),
            "rainbow" => rainbow(),
            _ => default_theme(),
        }
    }
}

fn normalize_theme_name(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(' ', "_")
}

fn default_theme() -> Theme {
    Theme {
        name: "default",
        title: Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        title_border: Style::new().fg(Color::Magenta),
        header: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ok: Style::new().fg(Color::Green),
        warn: Style::new().fg(Color::Yellow),
        error: Style::new().fg(Color::Red),
        dim: Style::new().fg(Color::Gray),
        strong: Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        selected: Style::new().bg(Color::DarkGray).add_modifier(Modifier::BOLD),
        key: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        border_error: Style::new().fg(Color::Red),
        tab_active: Style::new().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        tab_inactive: Style::new().fg(Color::Gray),
        gauge: Style::new().fg(Color::Cyan),
        status_left: Style::new().fg(Color::White).bg(Color::Rgb(40, 40, 40)),
        status_fill: Style::new().bg(Color::Rgb(40, 40, 40)),
        status_right: Style::new().fg(Color::Gray).bg(Color::Rgb(40, 40, 40)),
        danger_bold: Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    }
}

fn terminal() -> Theme {
    let mut theme = default_theme();
    theme.name = "terminal";
    theme.status_left = Style::new().fg(Color::White).bg(Color::Black);
    theme.status_fill = Style::new().bg(Color::Black);
    theme.status_right = Style::new().fg(Color::Gray).bg(Color::Black);
    theme
}

fn high_contrast() -> Theme {
    Theme {
        name: "high_contrast",
        title: Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        title_border: Style::new().fg(Color::Yellow),
        header: Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ok: Style::new().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
        warn: Style::new().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
        error: Style::new().fg(Color::LightRed).add_modifier(Modifier::BOLD),
        dim: Style::new().fg(Color::White),
        strong: Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        selected: Style::new().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
        key: Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
        border_error: Style::new().fg(Color::LightRed),
        tab_active: Style::new().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
        tab_inactive: Style::new().fg(Color::White),
        gauge: Style::new().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
        status_left: Style::new().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD),
        status_fill: Style::new().bg(Color::White),
        status_right: Style::new().fg(Color::Black).bg(Color::White),
        danger_bold: Style::new().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
    }
}

fn no_color() -> Theme {
    Theme {
        name: "no_color",
        title: Style::new().add_modifier(Modifier::BOLD),
        title_border: Style::new(),
        header: Style::new().add_modifier(Modifier::BOLD),
        ok: Style::new(),
        warn: Style::new().add_modifier(Modifier::BOLD),
        error: Style::new().add_modifier(Modifier::BOLD),
        dim: Style::new(),
        strong: Style::new().add_modifier(Modifier::BOLD),
        selected: Style::new().add_modifier(Modifier::REVERSED),
        key: Style::new().add_modifier(Modifier::BOLD),
        border_error: Style::new(),
        tab_active: Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD),
        tab_inactive: Style::new(),
        gauge: Style::new().add_modifier(Modifier::BOLD),
        status_left: Style::new().add_modifier(Modifier::REVERSED),
        status_fill: Style::new().add_modifier(Modifier::REVERSED),
        status_right: Style::new().add_modifier(Modifier::REVERSED),
        danger_bold: Style::new().add_modifier(Modifier::BOLD),
    }
}

fn rainbow() -> Theme {
    Theme {
        name: "rainbow",
        title: Style::new().fg(Color::Rgb(255, 64, 196)).add_modifier(Modifier::BOLD),
        title_border: Style::new().fg(Color::Rgb(176, 64, 255)),
        header: Style::new().fg(Color::Rgb(0, 224, 255)).add_modifier(Modifier::BOLD),
        ok: Style::new().fg(Color::Rgb(0, 255, 144)).add_modifier(Modifier::BOLD),
        warn: Style::new().fg(Color::Rgb(255, 220, 0)).add_modifier(Modifier::BOLD),
        error: Style::new().fg(Color::Rgb(255, 80, 128)).add_modifier(Modifier::BOLD),
        dim: Style::new().fg(Color::Rgb(176, 176, 220)),
        strong: Style::new().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD),
        selected: Style::new()
            .fg(Color::Rgb(255, 255, 255))
            .bg(Color::Rgb(108, 0, 255))
            .add_modifier(Modifier::BOLD),
        key: Style::new().fg(Color::Rgb(255, 148, 0)).add_modifier(Modifier::BOLD),
        border_error: Style::new().fg(Color::Rgb(255, 80, 128)),
        tab_active: Style::new()
            .fg(Color::Rgb(0, 0, 0))
            .bg(Color::Rgb(0, 224, 255))
            .add_modifier(Modifier::BOLD),
        tab_inactive: Style::new().fg(Color::Rgb(255, 128, 220)),
        gauge: Style::new().fg(Color::Rgb(0, 255, 220)).add_modifier(Modifier::BOLD),
        status_left: Style::new().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(88, 0, 128)),
        status_fill: Style::new().bg(Color::Rgb(88, 0, 128)),
        status_right: Style::new().fg(Color::Rgb(0, 255, 220)).bg(Color::Rgb(88, 0, 128)),
        danger_bold: Style::new().fg(Color::Rgb(255, 220, 0)).add_modifier(Modifier::BOLD),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_builtin_themes() {
        assert_eq!(Theme::from_config("default").name, "default");
        assert_eq!(Theme::from_config("terminal").name, "terminal");
        assert_eq!(Theme::from_config("system").name, "terminal");
        assert_eq!(Theme::from_config("high_contrast").name, "high_contrast");
        assert_eq!(Theme::from_config("no_color").name, "no_color");
        assert_eq!(Theme::from_config("rainbow").name, "rainbow");
    }

    #[test]
    fn falls_back_to_default_for_unknown_theme() {
        assert_eq!(Theme::from_config("unknown").name, "default");
    }
}
