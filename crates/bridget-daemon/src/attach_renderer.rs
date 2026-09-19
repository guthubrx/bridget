use pulldown_cmark::{Event, Options, Parser, Tag};
use std::io::Write;
use unicode_width::UnicodeWidthChar;

const SGR_RESET: &str = "\x1b[0m";
const SGR_HEADING: &str = "\x1b[1;38;5;45m";
const SGR_STRONG: &str = "\x1b[1m";
const SGR_EMPHASIS: &str = "\x1b[3m";
const SGR_INLINE_CODE: &str = "\x1b[38;5;214m";
const SGR_CODE_BLOCK: &str = "\x1b[38;5;223m\x1b[48;5;236m";
const SGR_QUOTE: &str = "\x1b[38;5;245m";
const SGR_AGENT: &str = "\x1b[1;38;5;45m";
const TAB_WIDTH: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Tone {
    #[default]
    Plain,
    Heading,
    InlineCode,
    CodeBlock,
    Quote,
    Agent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TextStyle {
    tone: Tone,
    strong: bool,
    emphasis: bool,
}

impl TextStyle {
    fn sgr(self) -> Option<&'static str> {
        match self.tone {
            Tone::Heading => Some(SGR_HEADING),
            Tone::InlineCode => Some(SGR_INLINE_CODE),
            Tone::CodeBlock => Some(SGR_CODE_BLOCK),
            Tone::Quote => Some(SGR_QUOTE),
            Tone::Agent => Some(SGR_AGENT),
            Tone::Plain if self.strong => Some(SGR_STRONG),
            Tone::Plain if self.emphasis => Some(SGR_EMPHASIS),
            Tone::Plain => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StyledSpan {
    text: String,
    style: TextStyle,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct StyledLine {
    spans: Vec<StyledSpan>,
    hard_break: bool,
    source_units: usize,
    synthetic_prefix_units: usize,
    continuation_indent: usize,
}

impl StyledLine {
    fn push(&mut self, text: impl AsRef<str>, style: TextStyle) {
        let text = text.as_ref();
        if text.is_empty() {
            return;
        }
        if let Some(last) = self.spans.last_mut()
            && last.style == style
        {
            last.text.push_str(text);
            return;
        }
        self.spans.push(StyledSpan {
            text: text.to_string(),
            style,
        });
    }

    fn push_char(&mut self, character: char, style: TextStyle) {
        if let Some(last) = self.spans.last_mut()
            && last.style == style
        {
            last.text.push(character);
            return;
        }
        self.spans.push(StyledSpan {
            text: character.to_string(),
            style,
        });
    }

    pub(super) fn plain(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }

    fn visible_units(&self) -> usize {
        self.source_units + usize::from(self.hard_break)
    }

    fn trim_visible_prefix(&self, mut units: usize) -> Option<Self> {
        if units >= self.source_units + usize::from(self.hard_break) {
            return None;
        }
        if units == 0 {
            return Some(self.clone());
        }
        let physical_units = units.saturating_add(self.synthetic_prefix_units);
        let mut trimmed = StyledLine {
            hard_break: self.hard_break,
            source_units: self.source_units.saturating_sub(units),
            ..StyledLine::default()
        };
        units = physical_units;
        for span in &self.spans {
            let span_units = span.text.chars().count();
            if units >= span_units {
                units -= span_units;
                continue;
            }
            trimmed.push(
                span.text.chars().skip(units).collect::<String>(),
                span.style,
            );
            units = 0;
        }
        Some(trimmed)
    }

    pub(super) fn write_to(&self, output: &mut impl Write, color: bool) {
        for span in &self.spans {
            if color && let Some(sgr) = span.style.sgr() {
                let _ = output.write_all(sgr.as_bytes());
                let _ = output.write_all(span.text.as_bytes());
                let _ = output.write_all(SGR_RESET.as_bytes());
            } else {
                let _ = output.write_all(span.text.as_bytes());
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Frame {
    Paragraph,
    Heading,
    Quote,
    CodeBlock,
    List,
    Item,
    Strong,
    Emphasis,
    Other,
}

#[derive(Clone, Copy, Debug)]
struct ListState {
    next: Option<u64>,
}

#[derive(Default)]
struct MarkdownBuilder {
    lines: Vec<StyledLine>,
    current: StyledLine,
    frames: Vec<Frame>,
    lists: Vec<ListState>,
    quote_depth: usize,
    code_depth: usize,
    heading_depth: usize,
    strong_depth: usize,
    emphasis_depth: usize,
    item_prefix: Option<String>,
    item_continuation: Option<String>,
}

impl MarkdownBuilder {
    fn start(&mut self, tag: Tag<'_>) {
        let frame = match tag {
            Tag::Paragraph => {
                self.finish_line(false);
                Frame::Paragraph
            }
            Tag::Heading { .. } => {
                self.finish_line(false);
                self.heading_depth += 1;
                Frame::Heading
            }
            Tag::BlockQuote(_) => {
                self.finish_line(false);
                self.quote_depth += 1;
                Frame::Quote
            }
            Tag::CodeBlock(_) => {
                self.finish_line(false);
                self.code_depth += 1;
                Frame::CodeBlock
            }
            Tag::List(start) => {
                self.finish_line(false);
                self.lists.push(ListState { next: start });
                Frame::List
            }
            Tag::Item => {
                self.finish_line(false);
                let depth = self.lists.len().saturating_sub(1);
                let marker = match self.lists.last_mut() {
                    Some(ListState { next: Some(next) }) => {
                        let marker = format!("{next}. ");
                        *next = next.saturating_add(1);
                        marker
                    }
                    _ => "• ".to_string(),
                };
                let indent = "  ".repeat(depth);
                let continuation_width = indent
                    .chars()
                    .chain(marker.chars())
                    .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
                    .sum();
                self.item_continuation = Some(" ".repeat(continuation_width));
                self.item_prefix = Some(format!("{indent}{marker}"));
                Frame::Item
            }
            Tag::Strong => {
                self.strong_depth += 1;
                Frame::Strong
            }
            Tag::Emphasis => {
                self.emphasis_depth += 1;
                Frame::Emphasis
            }
            _ => Frame::Other,
        };
        self.frames.push(frame);
    }

    fn end(&mut self) {
        let Some(frame) = self.frames.pop() else {
            return;
        };
        match frame {
            Frame::Paragraph => self.finish_line(false),
            Frame::Heading => {
                self.finish_line(false);
                self.heading_depth = self.heading_depth.saturating_sub(1);
            }
            Frame::Quote => {
                self.finish_line(false);
                self.quote_depth = self.quote_depth.saturating_sub(1);
            }
            Frame::CodeBlock => {
                self.finish_line(false);
                self.code_depth = self.code_depth.saturating_sub(1);
            }
            Frame::List => {
                self.finish_line(false);
                self.lists.pop();
            }
            Frame::Item => {
                self.finish_line(false);
                self.item_prefix = None;
                self.item_continuation = None;
            }
            Frame::Strong => self.strong_depth = self.strong_depth.saturating_sub(1),
            Frame::Emphasis => self.emphasis_depth = self.emphasis_depth.saturating_sub(1),
            Frame::Other => {}
        }
    }

    fn style(&self, inline_code: bool) -> TextStyle {
        let tone = if self.code_depth > 0 {
            Tone::CodeBlock
        } else if inline_code {
            Tone::InlineCode
        } else if self.heading_depth > 0 {
            Tone::Heading
        } else if self.quote_depth > 0 {
            Tone::Quote
        } else {
            Tone::Plain
        };
        TextStyle {
            tone,
            strong: self.strong_depth > 0,
            emphasis: self.emphasis_depth > 0,
        }
    }

    fn ensure_prefix(&mut self) {
        if !self.current.spans.is_empty() {
            return;
        }
        if self.quote_depth > 0 {
            self.current.push(
                "│ ".repeat(self.quote_depth),
                TextStyle {
                    tone: Tone::Quote,
                    ..TextStyle::default()
                },
            );
        }
        if self.code_depth > 0 {
            self.current.push(
                "  ",
                TextStyle {
                    tone: Tone::CodeBlock,
                    ..TextStyle::default()
                },
            );
        }
        if let Some(prefix) = self.item_prefix.take() {
            self.current.push(prefix, TextStyle::default());
            self.current.continuation_indent = line_width(&self.current);
        } else if let Some(continuation) = self.item_continuation.clone() {
            self.current.push(continuation, TextStyle::default());
            self.current.continuation_indent = line_width(&self.current);
        }
    }

    fn text(&mut self, value: &str, inline_code: bool) {
        let style = self.style(inline_code);
        // CommonMark décode notamment les entités numériques. Une entité telle
        // que `&#27;` peut donc redevenir ESC après l'assainissement pré-parser :
        // neutraliser une seconde fois chaque fragment effectivement émis.
        let safe = super::sanitize_data(value, super::MAX_TURN_RENDERED_CHARS);
        let mut parts = safe.text.split('\n').peekable();
        while let Some(part) = parts.next() {
            if !part.is_empty() {
                self.ensure_prefix();
                self.current.push(part, style);
            }
            if parts.peek().is_some() {
                self.finish_line(true);
            }
        }
    }

    fn soft_break(&mut self) {
        self.ensure_prefix();
        self.current.push(" ", self.style(false));
    }

    fn hard_break(&mut self) {
        self.finish_line(true);
    }

    fn rule(&mut self) {
        self.finish_line(false);
        self.current.push(
            "────────",
            TextStyle {
                tone: Tone::Quote,
                ..TextStyle::default()
            },
        );
        self.finish_line(true);
    }

    fn task_marker(&mut self, checked: bool) {
        self.ensure_prefix();
        self.current
            .push(if checked { "[x] " } else { "[ ] " }, TextStyle::default());
    }

    fn finish_line(&mut self, force: bool) {
        if force || !self.current.spans.is_empty() {
            self.lines.push(std::mem::take(&mut self.current));
        }
    }

    fn finish(mut self) -> Vec<StyledLine> {
        self.finish_line(false);
        self.lines
    }
}

pub(super) fn render_markdown(markdown: &str, columns: usize) -> Vec<StyledLine> {
    let normalized = markdown.replace("\r\n", "\n").replace('\r', "\n");
    let sanitized = super::sanitize_data(&normalized, super::MAX_TURN_RENDERED_CHARS);
    let mut builder = MarkdownBuilder::default();
    for event in Parser::new_ext(&sanitized.text, Options::empty()) {
        match event {
            Event::Start(tag) => builder.start(tag),
            Event::End(_) => builder.end(),
            Event::Text(text) => builder.text(&text, false),
            Event::Code(code) => builder.text(&code, true),
            Event::InlineMath(math) | Event::DisplayMath(math) => builder.text(&math, true),
            Event::Html(html) | Event::InlineHtml(html) => builder.text(&html, false),
            Event::FootnoteReference(reference) => {
                builder.text(&format!("[{reference}]"), false);
            }
            Event::SoftBreak => builder.soft_break(),
            Event::HardBreak => builder.hard_break(),
            Event::Rule => builder.rule(),
            Event::TaskListMarker(checked) => builder.task_marker(checked),
        }
    }
    if sanitized.truncated {
        builder.hard_break();
        builder.text("… [affichage tronqué]", false);
    }
    wrap_lines(builder.finish(), columns)
}

pub(super) fn agent_header(label: &str, columns: usize) -> Vec<StyledLine> {
    let mut line = StyledLine::default();
    line.push(
        format!("{label} ›"),
        TextStyle {
            tone: Tone::Agent,
            ..TextStyle::default()
        },
    );
    wrap_lines(vec![line], columns)
}

pub(super) fn plain_lines(lines: &[String], columns: usize) -> Vec<StyledLine> {
    let lines = lines
        .iter()
        .flat_map(|line| line.split('\n'))
        .map(|line| {
            let mut styled = StyledLine::default();
            styled.push(line, TextStyle::default());
            styled
        })
        .collect();
    wrap_lines(lines, columns)
}

pub(super) fn write_lines(output: &mut impl Write, lines: &[StyledLine], color: bool) {
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            let _ = output.write_all(b"\r\n");
        }
        line.write_to(output, color);
    }
}

pub(super) fn visible_units(lines: &[StyledLine]) -> usize {
    lines.iter().map(StyledLine::visible_units).sum()
}

pub(super) fn skip_visible_prefix(lines: &[StyledLine], mut units: usize) -> Vec<StyledLine> {
    let mut visible = Vec::new();
    for line in lines {
        let line_units = line.visible_units();
        if units >= line_units {
            units -= line_units;
            continue;
        }
        if let Some(line) = line.trim_visible_prefix(units) {
            visible.push(line);
        }
        units = 0;
    }
    visible
}

fn wrap_lines(lines: Vec<StyledLine>, columns: usize) -> Vec<StyledLine> {
    let columns = columns.max(1);
    let mut rows = Vec::new();
    for line in lines {
        let hard_wrap = line
            .spans
            .iter()
            .any(|span| span.style.tone == Tone::CodeBlock);
        let continuation_indent = (!hard_wrap)
            .then_some(line.continuation_indent)
            .filter(|indent| *indent > 0)
            .unwrap_or(0);
        let mut atoms = Vec::new();
        let mut source_width = 0usize;
        for span in line.spans {
            for character in span.text.chars() {
                if character == '\n' {
                    continue;
                }
                if character == '\t' {
                    let spaces = TAB_WIDTH - (source_width % TAB_WIDTH);
                    for _ in 0..spaces {
                        atoms.push((' ', span.style));
                        source_width += 1;
                    }
                    continue;
                }
                source_width += UnicodeWidthChar::width(character).unwrap_or(0);
                atoms.push((character, span.style));
            }
        }
        if atoms.is_empty() {
            rows.push(StyledLine {
                hard_break: true,
                source_units: 0,
                ..StyledLine::default()
            });
            continue;
        }
        let mut start = 0usize;
        let mut first_row = true;
        while start < atoms.len() {
            let synthetic_indent = if first_row {
                0
            } else {
                continuation_indent.min(columns.saturating_sub(1))
            };
            let available_columns = columns.saturating_sub(synthetic_indent).max(1);
            let remaining = &atoms[start..];
            let hard_split = hard_split_at(remaining, available_columns);
            if hard_split == remaining.len() {
                let source_units = remaining.len();
                rows.push(line_from_atoms(
                    remaining,
                    true,
                    source_units,
                    synthetic_indent,
                ));
                break;
            }
            let split = if hard_wrap {
                hard_split
            } else {
                word_split_at(remaining, hard_split)
            };
            let end = start + split.max(1).min(remaining.len());
            let mut row_end = end;
            let mut next_start = end;
            if !hard_wrap {
                while row_end > start && atoms[row_end - 1].0.is_whitespace() {
                    row_end -= 1;
                }
                while next_start < atoms.len() && atoms[next_start].0.is_whitespace() {
                    next_start += 1;
                }
            }
            rows.push(line_from_atoms(
                &atoms[start..row_end],
                false,
                next_start - start,
                synthetic_indent,
            ));
            start = next_start;
            first_row = false;
        }
    }
    rows
}

fn word_split_at(atoms: &[(char, TextStyle)], hard_split: usize) -> usize {
    atoms[..hard_split]
        .iter()
        .rposition(|(character, _)| character.is_whitespace())
        .filter(|index| *index > 0)
        .unwrap_or(hard_split)
}

fn hard_split_at(atoms: &[(char, TextStyle)], columns: usize) -> usize {
    let mut width = 0usize;
    for (index, (character, _)) in atoms.iter().enumerate() {
        let next = width.saturating_add(UnicodeWidthChar::width(*character).unwrap_or(0));
        if index > 0 && next > columns {
            return index;
        }
        width = next;
    }
    atoms.len()
}

fn line_from_atoms(
    atoms: &[(char, TextStyle)],
    hard_break: bool,
    source_units: usize,
    synthetic_indent: usize,
) -> StyledLine {
    let mut line = StyledLine {
        hard_break,
        source_units,
        synthetic_prefix_units: synthetic_indent,
        ..StyledLine::default()
    };
    if synthetic_indent > 0 {
        line.push(" ".repeat(synthetic_indent), TextStyle::default());
    }
    for (character, style) in atoms {
        line.push_char(*character, *style);
    }
    line
}

fn line_width(line: &StyledLine) -> usize {
    line.spans
        .iter()
        .flat_map(|span| span.text.chars())
        .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display_width(value: &str) -> usize {
        value
            .chars()
            .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
            .sum()
    }

    fn plain(lines: &[StyledLine]) -> String {
        lines
            .iter()
            .map(StyledLine::plain)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn colored(lines: &[StyledLine]) -> String {
        let mut output = Vec::new();
        for line in lines {
            line.write_to(&mut output, true);
        }
        String::from_utf8(output).unwrap()
    }

    fn without_local_styles(mut rendered: String) -> String {
        for sgr in [
            SGR_RESET,
            SGR_HEADING,
            SGR_STRONG,
            SGR_EMPHASIS,
            SGR_INLINE_CODE,
            SGR_CODE_BLOCK,
            SGR_QUOTE,
            SGR_AGENT,
        ] {
            rendered = rendered.replace(sgr, "");
        }
        rendered
    }

    #[test]
    fn spec093_markdown_complet_devient_une_vue_terminale_structuree() {
        let markdown = "# Titre\n\n- premier **fort**\n- second *accent*\n\n> citation\n\n`inline`\n\n```rust\nfn main() {\n    println!(\"ok\");\n}\n```";
        let lines = render_markdown(markdown, 80);
        let plain = plain(&lines);

        assert!(plain.contains("Titre"));
        assert!(plain.contains("• premier fort"));
        assert!(plain.contains("• second accent"));
        assert!(plain.contains("│ citation"));
        assert!(plain.contains("fn main() {"));
        assert!(!plain.contains("```"));
        assert!(!plain.contains("**"));
        assert!(!plain.contains("*accent*"));
        assert!(colored(&lines).contains("\x1b["));
    }

    #[test]
    fn spec093_fragments_convergent_vers_le_meme_markdown_final() {
        let fragments = ["## Ré", "ponse\n\n- un\n", "- deux avec `code`"];
        let mut accumulated = String::new();
        for fragment in fragments {
            accumulated.push_str(fragment);
            let _ = render_markdown(&accumulated, 24);
        }

        assert_eq!(
            render_markdown(&accumulated, 24),
            render_markdown("## Réponse\n\n- un\n- deux avec `code`", 24)
        );
    }

    #[test]
    fn spec093_unicode_et_ansi_hostile_ne_faussent_pas_la_largeur() {
        let lines = render_markdown(
            "**界界🙂** \u{1b}[31mrouge &#27;[2J &#x1b;]8;;https://x&#7;lien",
            8,
        );
        assert!(lines.iter().all(|line| display_width(&line.plain()) <= 8));
        let plain = plain(&lines);
        assert!(plain.contains("␛[31m"));
        assert!(plain.contains("␛[2J"));
        assert!(plain.contains("␛]8;;"));
        let unwrapped = plain.replace('\n', "");
        assert!(unwrapped.contains("␇lien"), "texte assaini réel: {plain:?}");
        assert!(!plain.contains('\u{1b}'));
        assert!(!plain.contains('\u{7}'));
        let colored = without_local_styles(colored(&lines));
        assert!(!colored.contains('\u{1b}'));
        assert!(!colored.contains('\u{7}'));
    }

    #[test]
    fn spec093_frontiere_logique_reste_stable_quand_la_largeur_change() {
        let markdown = format!("# Début\n\n{}\n\nFin", "mot lisible ".repeat(30));
        let wide = render_markdown(&markdown, 100);
        let narrow = render_markdown(&markdown, 45);
        assert_eq!(visible_units(&wide), visible_units(&narrow));

        let committed = visible_units(&wide[..2]);
        let tail = skip_visible_prefix(&narrow, committed);
        assert_eq!(
            visible_units(&tail),
            visible_units(&narrow).saturating_sub(committed)
        );
        assert!(plain(&tail).contains("Fin"));
    }

    #[test]
    fn spec093_listes_repliees_alignent_leur_suite_sans_deplacer_la_frontiere() {
        let markdown = "- une continuation très longue reste alignée sous le contenu\n\n12. autre continuation très longue";
        let wide = render_markdown(markdown, 80);
        let narrow = render_markdown(markdown, 22);
        let plain = narrow.iter().map(StyledLine::plain).collect::<Vec<_>>();

        assert!(plain[0].starts_with("• "));
        assert_eq!(plain[1].chars().take_while(|char| *char == ' ').count(), 2);
        let numbered = plain
            .iter()
            .position(|line| line.starts_with("12. "))
            .unwrap();
        assert_eq!(
            plain[numbered + 1]
                .chars()
                .take_while(|char| *char == ' ')
                .count(),
            4
        );
        assert_eq!(visible_units(&wide), visible_units(&narrow));
        assert!(narrow.iter().all(|line| display_width(&line.plain()) <= 22));
    }

    #[test]
    fn spec093_repli_64_kio_saute_les_espaces_sans_recopier_les_suffixes() {
        let markdown = format!("début {}fin", " ".repeat(64 * 1024));
        let rendered = render_markdown(&markdown, 45);

        assert!(plain(&rendered).contains("début"));
        assert!(plain(&rendered).contains("fin"));
        assert!(
            rendered
                .iter()
                .all(|line| display_width(&line.plain()) <= 45)
        );
    }
}
