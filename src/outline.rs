use ttf_parser::Face;
use ttf_parser::Rect;

use crate::segment::SegmentType;
use crate::segment::Segment;

struct Builder(Vec<Segment>);

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        let seg:Segment = Segment {
            segment_type: SegmentType::Origin,
            x_start: x, y_start: y,
            x1: 0f32, y1: 0f32,
            x2: 0f32, y2: 0f32,
            x_end: x, y_end: y

        };
        self.0.push(seg);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let last = self.0.get(self.0.len() - 1).unwrap();
        let x_last = last.x_end;
        let y_last = last.y_end;
        let seg:Segment = Segment {
            segment_type: SegmentType::Line,
            x_start: x_last, y_start:y_last,
            x1: 0f32, y1: 0f32,
            x2: 0f32, y2: 0f32,
            x_end: x, y_end: y

        };
        self.0.push(seg);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let last = self.0.get(self.0.len() - 1).unwrap();
        let x_last = last.x_end;
        let y_last = last.y_end;
        let seg:Segment = Segment {
            segment_type: SegmentType::Quad,
            x_start: x_last, y_start:y_last,
            x1: x1, y1: y1,
            x2: 0f32, y2: 0f32,
            x_end: x, y_end: y

        };
        self.0.push(seg);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let last = self.0.get(self.0.len() - 1).unwrap();
        let x_last = last.x_end;
        let y_last = last.y_end;
        let seg:Segment = Segment {
            segment_type: SegmentType::Cubic,
            x_start: x_last, y_start:y_last,
            x1: x1, y1: y1,
            x2: x2, y2: y2,
            x_end: x, y_end: y

        };
        self.0.push(seg);
    }

    fn close(&mut self) {
        let seg:Segment = Segment {
            segment_type: SegmentType::Close,
            x_start: 0f32, y_start: 0f32,
            x1: 0f32, y1: 0f32,
            x2: 0f32, y2: 0f32,
            x_end: 0f32, y_end: 0f32

        };
        self.0.push(seg);
    }
}

pub fn extract_outline(face:&Face, code_point:char) -> (Vec::<Segment>, Option<Rect>) {
    /* Given a face (parsed form of font file), returns a tuple containing the segments
    the segments of the face and its bounding box. The given code point can be the char
    of a character or a unicode symbol. */
    let glyph_id = face.glyph_index(code_point).unwrap();
    let mut glyph_builder = Builder(Vec::<Segment>::new());
    let bbox = face.outline_glyph(glyph_id, &mut glyph_builder);
    return (glyph_builder.0, bbox);
}