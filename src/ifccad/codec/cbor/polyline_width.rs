use super::*;
record!(IfccadPolylineVertexWidthOverride;
    0 => start: Option<f64> = Some(None),
    1 => end: Option<f64> = Some(None),
);
impl CborValue for IfccadPlanarPolylineWidths {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Constant"), (1, "PerVertex"), (2, "Defaulted")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(2)?;
        w.u64(0)?;
        w.u64(match self {
            Self::Constant { .. } => 0,
            Self::PerVertex { .. } => 1,
            Self::Defaulted { .. } => 2,
        })?;
        w.u64(1)?;
        match self {
            Self::Constant { width } => {
                w.map(1)?;
                w.u64(0)?;
                width.encode(w)
            }
            Self::PerVertex { widths } => {
                w.map(1)?;
                w.u64(0)?;
                widths.encode(w)
            }
            Self::Defaulted {
                default_start_width,
                default_end_width,
                overrides,
            } => {
                w.map(3)?;
                w.u64(0)?;
                default_start_width.encode(w)?;
                w.u64(1)?;
                default_end_width.encode(w)?;
                w.u64(2)?;
                overrides.encode(w)
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        f.finish()?;
        let names: &'static [(u64, &'static str)] = match code {
            0 => &[(0, "width")],
            1 => &[(0, "widths")],
            2 => &[
                (0, "default_start_width"),
                (1, "default_end_width"),
                (2, "overrides"),
            ],
            _ => return Err(r.error("unknown polyline width mode")),
        };
        let mut b = RecordFields::new(&body, names)?;
        let out = match code {
            0 => Self::Constant {
                width: b.take(0, None)?,
            },
            1 => Self::PerVertex {
                widths: b.take(0, None)?,
            },
            _ => Self::Defaulted {
                default_start_width: b.take(0, None)?,
                default_end_width: b.take(1, None)?,
                overrides: b.take(2, None)?,
            },
        };
        b.finish()?;
        Ok(out)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Constant { width: a }, Self::Constant { width: b }) => a.exact(b),
            (Self::PerVertex { widths: a }, Self::PerVertex { widths: b }) => a.exact(b),
            (
                Self::Defaulted {
                    default_start_width: a,
                    default_end_width: b,
                    overrides: c,
                },
                Self::Defaulted {
                    default_start_width: x,
                    default_end_width: y,
                    overrides: z,
                },
            ) => a.exact(x) && b.exact(y) && c.exact(z),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn width_float_slots_and_override_presence_are_strict() {
        // Integer, half and single precision cannot enter a binary64 width slot.
        for scalar in [vec![0], vec![0xf9, 0, 0], vec![0xfa, 0, 0, 0, 0]] {
            let mut bytes = vec![0xa2, 0, 0, 1, 0xa1, 0];
            bytes.extend(scalar);
            assert!(IfccadPlanarPolylineWidths::decode(&mut CborReader::new(
                &bytes,
                0,
                "/widths".into()
            ))
            .is_err());
        }
        // Duplicate zero start keys and null values never mean absent leaves.
        for bytes in [
            &[
                0xa2, 0, 0xfb, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xfb, 0, 0, 0, 0, 0, 0, 0, 0,
            ][..],
            &[0xa1, 0, 0xf6][..],
            &[0xa1, 2, 0xfb, 0, 0, 0, 0, 0, 0, 0, 0][..],
        ] {
            assert!(
                IfccadPolylineVertexWidthOverride::decode(&mut CborReader::new(
                    bytes,
                    0,
                    "/override".into()
                ))
                .is_err()
            );
        }
    }
}
