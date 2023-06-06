use std::str::FromStr;

use typst::util::option_eq;

use super::{
    Counter, CounterKey, HeadingElem, LocalName, Numbering, NumberingPattern, Refable,
};
use crate::layout::{BoxElem, HElem, HideElem, ParbreakElem, RepeatElem, Spacing};
use crate::prelude::*;
use crate::text::{LinebreakElem, SpaceElem, TextElem};

/// A table of contents, figures, or other elements.
///
/// This function generates a list of all occurrences of an element in the
/// document, up to a given depth. The element's numbering and page number will
/// be displayed in the outline alongside its title or caption. By default this
/// generates a table of contents.
///
/// ## Example { #example }
/// ```example
/// #outline()
///
/// = Introduction
/// #lorem(5)
///
/// = Prior work
/// #lorem(10)
/// ```
///
/// ## Alternative outlines { #alternative-outlines }
/// By setting the `target` parameter, the outline can be used to generate a
/// list of other kinds of elements than headings. In the example below, we list
/// all figures containing images by setting `target` to `{figure.where(kind:
/// image)}`. We could have also set it to just `figure`, but then the list
/// would also include figures containing tables or other material. For more
/// details on the `where` selector, [see here]($type/content.where).
///
/// ```example
/// #outline(
///   title: [List of Figures],
///   target: figure.where(kind: image),
/// )
///
/// #figure(
///   image("tiger.jpg"),
///   caption: [A nice figure!],
/// )
/// ```
///
/// Display: Outline
/// Category: meta
/// Keywords: Table of Contents
#[element(Show, Finalize, LocalName)]
pub struct OutlineElem {
    /// The title of the outline.
    ///
    /// - When set to `{auto}`, an appropriate title for the
    ///   [text language]($func/text.lang) will be used. This is the default.
    /// - When set to `{none}`, the outline will not have a title.
    /// - A custom title can be set by passing content.
    ///
    /// The outline's heading will not be numbered by default, but you can
    /// force it to be with a show-set rule:
    /// `{show outline: set heading(numbering: "1.")}`
    /// ```
    #[default(Some(Smart::Auto))]
    pub title: Option<Smart<Content>>,

    /// The type of element to include in the outline.
    ///
    /// To list figures containing a specific kind of element, like a table, you
    /// can write `{figure.where(kind: table)}`.
    ///
    /// ```example
    /// #outline(
    ///   title: [List of Tables],
    ///   target: figure.where(kind: table),
    /// )
    ///
    /// #figure(
    ///   table(
    ///     columns: 4,
    ///     [t], [1], [2], [3],
    ///     [y], [0.3], [0.7], [0.5],
    ///   ),
    ///   caption: [Experiment results],
    /// )
    /// ```
    #[default(LocatableSelector(Selector::Elem(
        HeadingElem::func(),
        Some(dict! { "outlined" => true })
    )))]
    pub target: LocatableSelector,

    /// The maximum level up to which elements are included in the outline. When
    /// this argument is `{none}`, all elements are included.
    ///
    /// ```example
    /// #set heading(numbering: "1.")
    /// #outline(depth: 2)
    ///
    /// = Yes
    /// Top-level section.
    ///
    /// == Still
    /// Subsection.
    ///
    /// === Nope
    /// Not included.
    /// ```
    pub depth: Option<NonZeroUsize>,

    /// How to indent the outline's entry lines. This defaults to `{none}`,
    /// which does not apply any indentation at all upon the outline's entries,
    /// which will then all be placed at the start of each line.
    ///
    /// If this option is set to `{auto}`, each entry (usually headings) will
    /// be indented enough to align with the last character of its parent's
    /// numbering. For example, if the parent entry is "3. Top heading" and the
    /// child entry is "3.1. Inner heading", the end result is that the child
    /// entry's line will begin where the "3." from the parent ends (after the
    /// last dot), but below. Consequently, if you specify `{auto}` indentation,
    /// you will only see a visible effect if a
    /// [heading numbering]($func/heading.numbering) is set for your headings
    /// (if using headings), or, in general, if your entries have some form of
    /// automatic numbering (generated by Typst) enabled.
    ///
    /// Note that specifying `{true}` (equivalent to `{auto}`) or `{false}`
    /// (equivalent to `{none}`) for this option is deprecated and will be
    /// removed in a future release. Please use `{none}` for no indentation
    /// or `{auto}` for automatic indentation instead.
    ///
    /// Alternatively, you may specify a custom indentation method, which
    /// doesn't depend on numbering. Setting this option to a length, such as
    /// `{2em}`, will indent each nested level by that much length, multiplied
    /// by the current nesting level (so a top-level heading, nested 0 times,
    /// wouldn't be indented at all; a heading nested once would be `{2em}`
    /// away from the start of the line, a heading nested twice would be
    /// `{4em}` away, and so on).
    ///
    /// It is also possible to set this option to a function, allowing for a
    /// more complete customization of the indentation. A function is expected
    /// to take a single parameter indcating the current nesting level
    /// (starting at `{0}` for top-level headings/elements), and return the
    /// indentation option for that level (or `{none}`). Such a function could
    /// be, for example, {n => n * 2em}` (indenting by `{2em}` times the
    /// nesting level), or `{n => [*!*] * n * n}` (indenting by a bold
    /// exclamation mark times the nesting level squared). Please note that the
    /// function is also called for nesting level 0, so be careful to not
    /// return a fixed value if you don't want to accidentally indent top-level
    /// entries by it (if that's not your intention), which you can solve by
    /// returning `{none}` when the received parameter is equal to `{0}`.
    ///
    ///
    /// ```example
    /// #set heading(numbering: "1.a.")
    ///
    /// #outline(title: "Contents (Automatic indentation)", indent: auto)
    /// #outline(title: "Contents (Length indentation)", indent: 2em)
    /// #outline(title: "Contents (Function indentation)", indent: n => [*!*] * n * n)
    ///
    /// = About ACME Corp.
    ///
    /// == History
    /// #lorem(10)
    ///
    /// == Products
    /// #lorem(10)
    /// ```
    #[default(None)]
    pub indent: Option<Smart<OutlineIndent>>,

    /// Content to fill the space between the title and the page number. Can be
    /// set to `none` to disable filling.
    ///
    /// ```example
    /// #outline(fill: line(length: 100%))
    ///
    /// = A New Beginning
    /// ```
    #[default(Some(RepeatElem::new(TextElem::packed(".")).pack()))]
    pub fill: Option<Content>,
}

impl Show for OutlineElem {
    #[tracing::instrument(name = "OutlineElem::show", skip_all)]
    fn show(&self, vt: &mut Vt, styles: StyleChain) -> SourceResult<Content> {
        let mut seq = vec![ParbreakElem::new().pack()];
        // Build the outline title.
        if let Some(title) = self.title(styles) {
            let title =
                title.unwrap_or_else(|| {
                    TextElem::packed(self.local_name(
                        TextElem::lang_in(styles),
                        TextElem::region_in(styles),
                    ))
                    .spanned(self.span())
                });

            seq.push(HeadingElem::new(title).with_level(NonZeroUsize::ONE).pack());
        }

        let indent = self.indent(styles);
        let depth = self.depth(styles).unwrap_or(NonZeroUsize::new(usize::MAX).unwrap());

        let mut ancestors: Vec<&Content> = vec![];
        let elems = vt.introspector.query(&self.target(styles).0);

        for elem in &elems {
            let Some(outlinable) = elem.with::<dyn Outlinable>() else {
                bail!(self.span(), "cannot outline {}", elem.func().name());
            };

            if depth < outlinable.level() {
                continue;
            }

            let Some(outline) = outlinable.outline(vt)? else {
                continue;
            };

            let location = elem.location().unwrap();

            // Deals with the ancestors of the current element.
            // This is only applicable for elements with a hierarchy/level.
            while ancestors
                .last()
                .and_then(|ancestor| ancestor.with::<dyn Outlinable>())
                .map_or(false, |last| last.level() >= outlinable.level())
            {
                ancestors.pop();
            }

            OutlineIndent::apply(&indent, vt, &ancestors, &mut seq, self.span())?;

            // Add the outline of the element.
            seq.push(outline.linked(Destination::Location(location)));

            let page_numbering = vt
                .introspector
                .page_numbering(location)
                .cast::<Option<Numbering>>()
                .unwrap()
                .unwrap_or_else(|| {
                    Numbering::Pattern(NumberingPattern::from_str("1").unwrap())
                });

            // Add filler symbols between the section name and page number.
            if let Some(filler) = self.fill(styles) {
                seq.push(SpaceElem::new().pack());
                seq.push(
                    BoxElem::new()
                        .with_body(Some(filler.clone()))
                        .with_width(Fr::one().into())
                        .pack(),
                );
                seq.push(SpaceElem::new().pack());
            } else {
                seq.push(HElem::new(Fr::one().into()).pack());
            }

            // Add the page number and linebreak.
            let page = Counter::new(CounterKey::Page)
                .at(location, vt)?
                .display(vt, &page_numbering)?;

            seq.push(page.linked(Destination::Location(location)));
            seq.push(LinebreakElem::new().pack());

            ancestors.push(elem);
        }

        seq.push(ParbreakElem::new().pack());

        Ok(Content::sequence(seq))
    }
}

impl Finalize for OutlineElem {
    fn finalize(&self, realized: Content, _: StyleChain) -> Content {
        realized
            .styled(HeadingElem::set_outlined(false))
            .styled(HeadingElem::set_numbering(None))
    }
}

impl LocalName for OutlineElem {
    fn local_name(&self, lang: Lang, region: Option<Region>) -> &'static str {
        match lang {
            Lang::ARABIC => "المحتويات",
            Lang::BOKMÅL => "Innhold",
            Lang::CHINESE if option_eq(region, "TW") => "目錄",
            Lang::CHINESE => "目录",
            Lang::CZECH => "Obsah",
            Lang::DANISH => "Indhold",
            Lang::DUTCH => "Inhoudsopgave",
            Lang::FRENCH => "Table des matières",
            Lang::GERMAN => "Inhaltsverzeichnis",
            Lang::ITALIAN => "Indice",
            Lang::NYNORSK => "Innhald",
            Lang::POLISH => "Spis treści",
            Lang::PORTUGUESE => "Sumário",
            Lang::RUSSIAN => "Содержание",
            Lang::SLOVENIAN => "Kazalo",
            Lang::SPANISH => "Índice",
            Lang::SWEDISH => "Innehåll",
            Lang::UKRAINIAN => "Зміст",
            Lang::VIETNAMESE => "Mục lục",
            Lang::ENGLISH | _ => "Contents",
        }
    }
}

/// Marks an element as being able to be outlined. This is used to implement the
/// `#outline()` element.
pub trait Outlinable: Refable {
    /// Produce an outline item for this element.
    fn outline(&self, vt: &mut Vt) -> SourceResult<Option<Content>>;

    /// Returns the nesting level of this element.
    fn level(&self) -> NonZeroUsize {
        NonZeroUsize::ONE
    }
}

#[derive(Debug, Clone)]
pub enum OutlineIndent {
    Bool(bool),
    Length(Spacing),
    Function(Func),
}

impl OutlineIndent {
    fn apply(
        indent: &Option<Smart<Self>>,
        vt: &mut Vt,
        ancestors: &Vec<&Content>,
        seq: &mut Vec<Content>,
        span: Span,
    ) -> SourceResult<()> {
        match &indent {
            // 'none' | 'false' => no indenting
            None | Some(Smart::Custom(OutlineIndent::Bool(false))) => {}

            // 'auto' | 'true' => use numbering alignment for indenting
            Some(Smart::Auto | Smart::Custom(OutlineIndent::Bool(true))) => {
                // Add hidden ancestors numberings to realize the indent.
                let mut hidden = Content::empty();
                for ancestor in ancestors {
                    let ancestor_outlinable = ancestor.with::<dyn Outlinable>().unwrap();

                    if let Some(numbering) = ancestor_outlinable.numbering() {
                        let numbers = ancestor_outlinable
                            .counter()
                            .at(ancestor.location().unwrap(), vt)?
                            .display(vt, &numbering)?;

                        hidden += numbers + SpaceElem::new().pack();
                    };
                }

                if !ancestors.is_empty() {
                    seq.push(HideElem::new(hidden).pack());
                    seq.push(SpaceElem::new().pack());
                }
            }

            // Length => indent with some fixed spacing per level
            Some(Smart::Custom(OutlineIndent::Length(length))) => {
                seq.push(HElem::new(*length).pack().repeat(ancestors.len()));
            }

            // Function => call function with the current depth and take
            // the returned content
            Some(Smart::Custom(OutlineIndent::Function(func))) => {
                let depth = ancestors.len();
                let returned = func.call_vt(vt, [depth])?;
                let Ok(returned) = returned.cast::<Content>() else {
                    bail!(
                        span,
                        "indent function must return content"
                    );
                };
                if !returned.is_empty() {
                    seq.push(returned);
                }
            }
        };

        Ok(())
    }
}

cast! {
    OutlineIndent,
    self => match self {
        Self::Bool(v) => v.into_value(),
        Self::Length(v) => v.into_value(),
        Self::Function(v) => v.into_value()
    },
    v: bool => OutlineIndent::Bool(v),
    v: Spacing => OutlineIndent::Length(v),
    v: Func => OutlineIndent::Function(v),
}
