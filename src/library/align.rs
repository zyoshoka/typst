use super::*;

/// `align`: Align content along the layouting axes.
///
/// Which axis an alignment should apply to (main or cross) is inferred from
/// either the argument itself (for anything other than `center`) or from the
/// second argument if present, defaulting to the cross axis for a single
/// `center` alignment.
///
/// # Positional arguments
/// - Alignments: variadic, of type `alignment`.
/// - Body:       optional, of type `template`.
///
/// # Named arguments
/// - Horizontal alignment: `horizontal`, of type `alignment`.
/// - Vertical alignment:   `vertical`, of type `alignment`.
///
/// # Relevant types and constants
/// - Type `alignment`
///     - `left`
///     - `right`
///     - `top`
///     - `bottom`
///     - `center`
pub fn align(ctx: &mut EvalContext, args: &mut ValueArgs) -> Value {
    let first = args.find(ctx);
    let second = args.find(ctx);
    let hor = args.get(ctx, "horizontal");
    let ver = args.get(ctx, "vertical");
    let body = args.find::<ValueTemplate>(ctx);

    Value::template("align", move |ctx| {
        let snapshot = ctx.state.clone();

        let values = first
            .into_iter()
            .chain(second.into_iter())
            .map(|arg: Spanned<AlignValue>| (arg.v.axis(), arg))
            .chain(hor.into_iter().map(|arg| (Some(SpecAxis::Horizontal), arg)))
            .chain(ver.into_iter().map(|arg| (Some(SpecAxis::Vertical), arg)));

        apply(ctx, values);

        if ctx.state.aligns.main != snapshot.aligns.main {
            ctx.push_linebreak();
        }

        if let Some(body) = &body {
            body.exec(ctx);
            ctx.state = snapshot;
        }
    })
}

/// Deduplicate and apply the alignments.
fn apply(
    ctx: &mut ExecContext,
    values: impl Iterator<Item = (Option<SpecAxis>, Spanned<AlignValue>)>,
) {
    let mut had = Gen::uniform(false);
    let mut had_center = false;

    for (axis, Spanned { v: arg, span }) in values {
        // Check whether we know which axis this alignment belongs to.
        if let Some(axis) = axis {
            // We know the axis.
            let gen_axis = axis.switch(ctx.state.dirs);
            let gen_align = arg.switch(ctx.state.dirs);

            if arg.axis().map_or(false, |a| a != axis) {
                ctx.diag(error!(span, "invalid alignment for {} axis", axis));
            } else if had.get(gen_axis) {
                ctx.diag(error!(span, "duplicate alignment for {} axis", axis));
            } else {
                *ctx.state.aligns.get_mut(gen_axis) = gen_align;
                *had.get_mut(gen_axis) = true;
            }
        } else {
            // We don't know the axis: This has to be a `center` alignment for a
            // positional argument.
            debug_assert_eq!(arg, AlignValue::Center);

            if had.main && had.cross {
                ctx.diag(error!(span, "duplicate alignment"));
            } else if had_center {
                // Both this and the previous one are unspecified `center`
                // alignments. Both axes should be centered.
                ctx.state.aligns.main = Align::Center;
                ctx.state.aligns.cross = Align::Center;
                had = Gen::uniform(true);
            } else {
                had_center = true;
            }
        }

        // If we we know the other alignment, we can handle the unspecified
        // `center` alignment.
        if had_center && (had.main || had.cross) {
            if had.main {
                ctx.state.aligns.cross = Align::Center;
            } else {
                ctx.state.aligns.main = Align::Center;
            }
            had = Gen::uniform(true);
            had_center = false;
        }
    }

    // If `had_center` wasn't flushed by now, it's the only argument and
    // then we default to applying it to the cross axis.
    if had_center {
        ctx.state.aligns.cross = Align::Center;
    }
}

/// An alignment value.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum AlignValue {
    Left,
    Center,
    Right,
    Top,
    Bottom,
}

impl AlignValue {
    /// The specific axis this alignment refers to.
    fn axis(self) -> Option<SpecAxis> {
        match self {
            Self::Left => Some(SpecAxis::Horizontal),
            Self::Right => Some(SpecAxis::Horizontal),
            Self::Top => Some(SpecAxis::Vertical),
            Self::Bottom => Some(SpecAxis::Vertical),
            Self::Center => None,
        }
    }
}

impl Switch for AlignValue {
    type Other = Align;

    fn switch(self, dirs: LayoutDirs) -> Self::Other {
        let get = |dir: Dir, at_positive_start| {
            if dir.is_positive() == at_positive_start {
                Align::Start
            } else {
                Align::End
            }
        };

        let dirs = dirs.switch(dirs);
        match self {
            Self::Left => get(dirs.horizontal, true),
            Self::Right => get(dirs.horizontal, false),
            Self::Top => get(dirs.vertical, true),
            Self::Bottom => get(dirs.vertical, false),
            Self::Center => Align::Center,
        }
    }
}

impl Display for AlignValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.pad(match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        })
    }
}

typify! {
    AlignValue: "alignment",
}
