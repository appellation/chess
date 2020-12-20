use crate::assets;
use chess::{Board, Color, Piece, Square};
use std::convert::TryFrom;

const WHITE_CHESS_KING: char = '\u{2654}';
const WHITE_CHESS_QUEEN: char = '\u{2655}';
const WHITE_CHESS_ROOK: char = '\u{2656}';
const WHITE_CHESS_BISHOP: char = '\u{2657}';
const WHITE_CHESS_KNIGHT: char = '\u{2658}';
const WHITE_CHESS_PAWN: char = '\u{2659}';

const BLACK_CHESS_KING: char = '\u{265A}';
const BLACK_CHESS_QUEEN: char = '\u{256B}';
const BLACK_CHESS_ROOK: char = '\u{265C}';
const BLACK_CHESS_BISHOP: char = '\u{265D}';
const BLACK_CHESS_KNIGHT: char = '\u{265E}';
const BLACK_CHESS_PAWN: char = '\u{265F}';

const ICON_SIZE: usize = 144;

pub struct Svg(usvg::Node);

impl TryFrom<Svg> for Vec<u8> {
	type Error = png::EncodingError;

	fn try_from(value: Svg) -> Result<Self, Self::Error> {
		let image = resvg::render_node(&value.0, usvg::FitTo::Original, Some(usvg::Color::white()))
			.unwrap();

		let mut w = vec![];
		{
			let mut encoder = png::Encoder::new(&mut w, image.width(), image.height());
			encoder.set_color(png::ColorType::RGBA);
			encoder.set_depth(png::BitDepth::Eight);
			let mut writer = encoder.write_header()?;
			writer.write_image_data(image.data())?;
		}

		Ok(w)
	}
}

impl TryFrom<Board> for Svg {
	type Error = usvg::Error;

	fn try_from(board: chess::Board) -> Result<Self, Self::Error> {
		let mut root = usvg::Tree::from_data(assets::BASE_BOARD, &Default::default())?.root();

		for square in board.combined().into_iter() {
			// safe to unwrap since the iterator is over valid squares
			let piece = board.piece_on(square).unwrap();
			let color = board.color_on(square).unwrap();

			let child_asset: &[u8] = ColoredPiece(piece, color).into();
			let child_root = usvg::Tree::from_data(child_asset, &Default::default())?.root();

			let group = usvg::Group {
				transform: usvg::Transform::new_translate(square.x(1152), square.y(1152)),
				..Default::default()
			};

			let mut group_node = usvg::Node::new(usvg::NodeKind::Group(group));
			group_node.append(child_root);
			root.append(group_node);
		}

		Ok(Self(root))
	}
}

trait Positioned {
	fn x(&self, size: usize) -> f64;
	fn y(&self, size: usize) -> f64;
}

impl Positioned for Square {
	fn x(&self, size: usize) -> f64 {
		self.get_file().to_index() as f64 / 8.0 * size as f64
	}

	fn y(&self, size: usize) -> f64 {
		(size as f64 / 8.0 * (8 - self.get_rank().to_index()) as f64) - ICON_SIZE as f64
	}
}

pub struct ColoredPiece(pub Piece, pub Color);

impl From<ColoredPiece> for char {
	fn from(pos: ColoredPiece) -> Self {
		match pos {
			ColoredPiece(Piece::King, Color::White) => WHITE_CHESS_KING,
			ColoredPiece(Piece::Queen, Color::White) => WHITE_CHESS_QUEEN,
			ColoredPiece(Piece::Rook, Color::White) => WHITE_CHESS_ROOK,
			ColoredPiece(Piece::Bishop, Color::White) => WHITE_CHESS_BISHOP,
			ColoredPiece(Piece::Knight, Color::White) => WHITE_CHESS_KNIGHT,
			ColoredPiece(Piece::Pawn, Color::White) => WHITE_CHESS_PAWN,
			ColoredPiece(Piece::King, Color::Black) => BLACK_CHESS_KING,
			ColoredPiece(Piece::Queen, Color::Black) => BLACK_CHESS_QUEEN,
			ColoredPiece(Piece::Rook, Color::Black) => BLACK_CHESS_ROOK,
			ColoredPiece(Piece::Bishop, Color::Black) => BLACK_CHESS_BISHOP,
			ColoredPiece(Piece::Knight, Color::Black) => BLACK_CHESS_KNIGHT,
			ColoredPiece(Piece::Pawn, Color::Black) => BLACK_CHESS_PAWN,
		}
	}
}

impl From<ColoredPiece> for &[u8] {
	fn from(pos: ColoredPiece) -> Self {
		match pos {
			ColoredPiece(Piece::King, Color::White) => assets::WHITE_KING,
			ColoredPiece(Piece::Queen, Color::White) => assets::WHITE_QUEEN,
			ColoredPiece(Piece::Rook, Color::White) => assets::WHITE_ROOK,
			ColoredPiece(Piece::Bishop, Color::White) => assets::WHITE_BISHOP,
			ColoredPiece(Piece::Knight, Color::White) => assets::WHITE_KNIGHT,
			ColoredPiece(Piece::Pawn, Color::White) => assets::WHITE_PAWN,
			ColoredPiece(Piece::King, Color::Black) => assets::BLACK_KING,
			ColoredPiece(Piece::Queen, Color::Black) => assets::BLACK_QUEEN,
			ColoredPiece(Piece::Rook, Color::Black) => assets::BLACK_ROOK,
			ColoredPiece(Piece::Bishop, Color::Black) => assets::BLACK_BISHOP,
			ColoredPiece(Piece::Knight, Color::Black) => assets::BLACK_KNIGHT,
			ColoredPiece(Piece::Pawn, Color::Black) => assets::BLACK_PAWN,
		}
	}
}
