use deku::bitvec::BitView;
use deku::prelude::*;

use crate::error::NuccError;

use crate::nucc_chunk::{Curve as ChunkCurve, CurveFormat, CurveHeader};

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct Vector2(f32, f32);

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct Vector3(f32, f32, f32);

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct Vector3Short(i16, i16, i16);

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct Quaternion(f32, f32, f32, f32);

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct QuaternionShort(i16, i16, i16, i16);

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct RGB(u8, u8, u8);

#[derive(Debug)]
pub enum Keyframes {
    None,
    Float(Vec<f32>),
    FloatLinear(Vec<(i32, f32)>),
    Vector2(Vec<Vector2>),
    Vector2Linear(Vec<(i32, Vector2)>),
    Vector3(Vec<Vector3>),
    Vector3Short(Vec<Vector3Short>),
    Vector3Linear(Vec<(i32, Vector3)>),
    Vector3ShortLinear(Vec<(i32, Vector3Short)>),
    Quaternion(Vec<Quaternion>),
    QuaternionShort(Vec<QuaternionShort>),
    QuaternionLinear(Vec<(i32, Quaternion)>),
    RGB(Vec<RGB>),
    Opacity(Vec<i16>),
}

impl Keyframes {
    pub fn keyframe_count(&self) -> usize {
        match self {
            Keyframes::Float(frames) => frames.len(),
            Keyframes::FloatLinear(frames) => frames.len(),
            Keyframes::Vector2(frames) => frames.len(),
            Keyframes::Vector2Linear(frames) => frames.len(),
            Keyframes::Vector3(frames) => frames.len(),
            Keyframes::Vector3Short(frames) => frames.len(),
            Keyframes::Vector3Linear(frames) => frames.len(),
            Keyframes::QuaternionShort(frames) => frames.len(),
            Keyframes::QuaternionLinear(frames) => frames.len(),
            Keyframes::RGB(frames) => frames.len(),
            Keyframes::Opacity(frames) => frames.len(),
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub struct Curve {
    pub channel: Channel,
    pub interp_type: InterpolationType,
    keyframes: Keyframes,
}

impl Curve {
    pub fn keyframes(&self) -> &Keyframes {
        &self.keyframes
    }

    pub fn keyframes_mut(&mut self) -> &mut Keyframes {
        &mut self.keyframes
    }

    pub fn set_keyframes(&mut self, keyframes: Keyframes) -> Result<(), NuccError> {
        // TODO: Add a message to the error
        let err = Err(NuccError::GenericError);
        let mut set_frames = |frames| {
            self.keyframes = frames;
            Ok(())
        };

        match self.channel {
            Channel::Location | Channel::Scale => match keyframes {
                Keyframes::Vector3(_)
                | Keyframes::Vector3Short(_)
                | Keyframes::Vector3Linear(_) => set_frames(keyframes),
                _ => err,
            },
            Channel::Rotation => match keyframes {
                Keyframes::Vector3(_)
                | Keyframes::Vector3Short(_)
                | Keyframes::Vector3Linear(_)
                | Keyframes::QuaternionShort(_)
                | Keyframes::QuaternionLinear(_) => set_frames(keyframes),
                _ => err,
            },
            Channel::Opacity => match keyframes {
                Keyframes::Float(_) | Keyframes::FloatLinear(_) | Keyframes::Opacity(_) => {
                    set_frames(keyframes)
                }
                _ => err,
            },
            Channel::Fov | Channel::Property => match keyframes {
                Keyframes::Float(_) | Keyframes::FloatLinear(_) => set_frames(keyframes),
                _ => err,
            },
            Channel::Color => match keyframes {
                Keyframes::RGB(_) => set_frames(keyframes),
                _ => err,
            },
        }
    }

    pub fn new(channel: Channel, interp_type: InterpolationType, keyframes: Keyframes) -> Self {
        let mut curve = Self {
            channel,
            interp_type,
            keyframes: Keyframes::None,
        };

        curve.set_keyframes(keyframes).unwrap();
        curve
    }
}

pub struct CurveChunkConverter(pub Channel, pub CurveHeader, pub ChunkCurve);

impl From<CurveChunkConverter> for Curve {
    fn from(converter: CurveChunkConverter) -> Self {
        let CurveChunkConverter(channel, header, chunk) = converter;
        
        let interp_type = match header.curve_format {
            CurveFormat::Vector3Fixed
            | CurveFormat::EulerXYZFixed
            | CurveFormat::FloatFixed
            | CurveFormat::Vector2Fixed => InterpolationType::None,

            CurveFormat::OpacityShortTable
            | CurveFormat::ScaleShortTable
            | CurveFormat::QuaternionShortTable
            | CurveFormat::ColorRGBTable
            | CurveFormat::Vector3Table
            | CurveFormat::FloatTable
            | CurveFormat::QuaternionTable => InterpolationType::None,

            CurveFormat::FloatTableNoInterp
            | CurveFormat::Vector3TableNoInterp
            | CurveFormat::QuaternionShortTableNoInterp
            | CurveFormat::OpacityShortTableNoInterp => InterpolationType::None,

            CurveFormat::Vector3Linear
            | CurveFormat::QuatnerionLinear
            | CurveFormat::FloatLinear
            | CurveFormat::Vector2Linear
            | CurveFormat::Vector3ShortLinear => InterpolationType::Linear,

            CurveFormat::Vector3Bezier => InterpolationType::Bezier,
            CurveFormat::EulerInterpolated => todo!(),
        };

        let endianness = deku::ctx::Endian::Big;
        let mut data = chunk.data.view_bits();

        let keyframes = match header.curve_format {
            CurveFormat::Vector3Fixed
            | CurveFormat::Vector3Table
            | CurveFormat::Vector3TableNoInterp
            | CurveFormat::EulerXYZFixed => Keyframes::Vector3({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, value) = Vector3::read(data, endianness).unwrap();
                    data = rest;

                    vec.push(value);
                }

                vec
            }),
            CurveFormat::Vector3Linear => Keyframes::Vector3Linear({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, frame) = i32::read(data, endianness).unwrap();
                    data = rest;
                    let (rest, value) = Vector3::read(data, endianness).unwrap();
                    data = rest;

                    vec.push((frame, value));
                }

                vec
            }),
            CurveFormat::Vector3Bezier => todo!(),
            CurveFormat::EulerInterpolated => todo!(),
            CurveFormat::QuatnerionLinear => Keyframes::QuaternionLinear({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, frame) = i32::read(data, endianness).unwrap();
                    data = rest;
                    let (rest, value) = Quaternion::read(data, endianness).unwrap();
                    data = rest;

                    vec.push((frame, value));
                }

                vec
            }),
            CurveFormat::FloatFixed | CurveFormat::FloatTable | CurveFormat::FloatTableNoInterp => {
                Keyframes::Float({
                    let mut vec = vec![];
                    vec.reserve_exact(header.frame_count as usize);
                    for _ in 0..header.frame_count {
                        let (rest, value) = f32::read(data, endianness).unwrap();
                        data = rest;

                        vec.push(value);
                    }

                    vec
                })
            }
            CurveFormat::FloatLinear => Keyframes::FloatLinear({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, frame) = i32::read(data, endianness).unwrap();
                    data = rest;
                    let (rest, value) = f32::read(data, endianness).unwrap();
                    data = rest;

                    vec.push((frame, value));
                }

                vec
            }),
            CurveFormat::Vector2Fixed => Keyframes::Vector2({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, value) = Vector2::read(data, endianness).unwrap();
                    data = rest;

                    vec.push(value);
                }

                vec
            }),
            CurveFormat::Vector2Linear => Keyframes::Vector2Linear({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, frame) = i32::read(data, endianness).unwrap();
                    data = rest;
                    let (rest, value) = Vector2::read(data, endianness).unwrap();
                    data = rest;

                    vec.push((frame, value));
                }

                vec
            }),
            CurveFormat::OpacityShortTable | CurveFormat::OpacityShortTableNoInterp => {
                Keyframes::Opacity({
                    let mut vec = vec![];
                    vec.reserve_exact(header.frame_count as usize);
                    for _ in 0..header.frame_count {
                        let (rest, value) = i16::read(data, endianness).unwrap();
                        data = rest;

                        vec.push(value);
                    }

                    vec
                })
            }
            CurveFormat::ScaleShortTable => Keyframes::Vector3Short({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, value) = Vector3Short::read(data, endianness).unwrap();
                    data = rest;

                    vec.push(value);
                }

                vec
            }),
            CurveFormat::QuaternionShortTable | CurveFormat::QuaternionShortTableNoInterp => {
                Keyframes::QuaternionShort({
                    let mut vec = vec![];
                    vec.reserve_exact(header.frame_count as usize);
                    for _ in 0..header.frame_count {
                        let (rest, value) = QuaternionShort::read(data, endianness).unwrap();
                        data = rest;

                        vec.push(value);
                    }

                    vec
                })
            }
            CurveFormat::ColorRGBTable => Keyframes::RGB({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, value) = RGB::read(data, endianness).unwrap();
                    data = rest;

                    vec.push(value);
                }

                vec
            }),
            CurveFormat::QuaternionTable => Keyframes::Quaternion({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, value) = Quaternion::read(data, endianness).unwrap();
                    data = rest;

                    vec.push(value);
                }

                vec
            }),
            CurveFormat::Vector3ShortLinear => Keyframes::Vector3ShortLinear({
                let mut vec = vec![];
                vec.reserve_exact(header.frame_count as usize);
                for _ in 0..header.frame_count {
                    let (rest, frame) = i32::read(data, ()).unwrap();
                    data = rest;
                    let (rest, value) = Vector3Short::read(data, endianness).unwrap();
                    data = rest;

                    vec.push((frame, value));
                }

                vec
            }),
        };

        Curve::new(channel, interp_type, keyframes)
    }
}

#[derive(Debug, Clone)] // Needed so we can use iter::repeat on Channel::Property for EntryFormat::Material
pub enum Channel {
    Location,
    Rotation,
    Scale,
    Opacity,
    Fov,
    Color,
    Property,
}

#[derive(Debug)]
pub enum InterpolationType {
    None,
    Linear,
    Bezier,
}
