use chess::{File, GameResult, Piece, Rank, Square};
use nom::{Err, IResult, branch::alt, bytes::complete::{tag, take}, combinator::{complete, map, map_res, opt, value}, error::{convert_error, VerboseError}, sequence::tuple};
use serde::{Deserialize, Serialize};
use std::{
	convert::{TryFrom, TryInto},
	fmt::{self, Display},
	str::FromStr,
};

#[derive(Debug, Clone)]
pub enum CheckState {
	None,
	Check,
	Double,
	Mate,
}

#[derive(Debug, Clone)]
pub enum CastleSide {
	King,
	Queen,
}

impl Display for CastleSide {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			CastleSide::King => "0-0",
			CastleSide::Queen => "0-0-0",
		})
	}
}

#[derive(Debug, Clone)]
pub enum EndOfGameState {
	None,
	WhiteWins,
	BlackWins,
	Draw,
}

impl Display for EndOfGameState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			EndOfGameState::None => "*",
			EndOfGameState::WhiteWins => "1-0",
			EndOfGameState::Draw => "½-½",
			EndOfGameState::BlackWins => "0-1",
		})
	}
}

impl From<GameResult> for EndOfGameState {
	fn from(result: GameResult) -> Self {
		match result {
			GameResult::BlackCheckmates | GameResult::WhiteResigns => Self::BlackWins,
			GameResult::WhiteCheckmates | GameResult::BlackResigns => Self::WhiteWins,
			GameResult::DrawAccepted | GameResult::DrawDeclared | GameResult::Stalemate => Self::Draw
		}
	}
}

impl From<Option<GameResult>> for EndOfGameState {
	fn from(maybe_result: Option<GameResult>) -> Self {
		match maybe_result {
			Some(result) => result.into(),
			None => EndOfGameState::None,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "&str", into = "String")]
pub enum SANChessMove {
	Move {
		piece: Piece,
		capture: bool,
		src_rank: Option<Rank>,
		src_file: Option<File>,
		dest: Square,
		promotion: Option<Piece>,
		check: CheckState,
	},
	Castle(CastleSide),
	OfferDraw,
	EOG(EndOfGameState),
}

#[derive(Debug, thiserror::Error)]
pub enum SANError {
	#[error("Parse error: {0}")]
	Parse(String),
}

fn parse_piece(input: &str) -> IResult<&str, Piece, VerboseError<&str>> {
	map(opt(alt((
		value(Piece::King, tag("K")),
		value(Piece::Bishop, tag("B")),
		value(Piece::Knight, tag("N")),
		value(Piece::Queen, tag("Q")),
		value(Piece::Rook, tag("R")),
	))), |maybe_piece| maybe_piece.unwrap_or(Piece::Pawn))(input)
}

fn parse_capture(input: &str) -> IResult<&str, bool, VerboseError<&str>> {
	map(opt(tag("x")), |capture| capture.is_some())(input)
}

fn parse_square(input: &str) -> IResult<&str, Square, VerboseError<&str>> {
	map_res(take(2usize), |s: &str| s.parse())(input)
}

fn parse_check_state(input: &str) -> IResult<&str, CheckState, VerboseError<&str>> {
	map(
		opt(alt((
			value(CheckState::Check, tag("+")),
			value(CheckState::Double, tag("++")),
			value(CheckState::Mate, tag("#")),
		))),
		|maybe_state| maybe_state.unwrap_or(CheckState::None),
	)(input)
}

fn parse_draw(input: &str) -> IResult<&str, SANChessMove, VerboseError<&str>> {
	value(SANChessMove::OfferDraw, tag("="))(input)
}

fn parse_castle_side(input: &str) -> IResult<&str, CastleSide, VerboseError<&str>> {
	alt((
		value(CastleSide::King, alt((tag("0-0"), tag("O-O")))),
		value(CastleSide::Queen, alt((tag("0-0-0"), tag("O-O-O")))),
	))(input)
}

fn parse_eog_state(input: &str) -> IResult<&str, EndOfGameState, VerboseError<&str>> {
	alt((
		value(EndOfGameState::None, tag("*")),
		value(EndOfGameState::BlackWins, tag("0-1")),
		value(EndOfGameState::Draw, alt((tag("½-½"), tag("1/2-1/2")))),
		value(EndOfGameState::WhiteWins, tag("1-0")),
	))(input)
}

fn parse_rank(input: &str) -> IResult<&str, Rank, VerboseError<&str>> {
	map(
		map_res(take(1usize), |c: &str| c.parse::<usize>()),
		|index| Rank::from_index(index - 1),
	)(input)
}

fn parse_file(input: &str) -> IResult<&str, File, VerboseError<&str>> {
	alt((
		value(File::A, tag("a")),
		value(File::B, tag("b")),
		value(File::C, tag("c")),
		value(File::D, tag("d")),
		value(File::E, tag("e")),
		value(File::F, tag("f")),
		value(File::G, tag("g")),
		value(File::H, tag("h")),
	))(input)
}

fn parse_disambiguator(input: &str) -> IResult<&str, (Option<File>, Option<Rank>), VerboseError<&str>> {
	alt((
		map(parse_file, |file| (Some(file), None)),
		map(parse_rank, |rank| (None, Some(rank))),
		map(tuple((parse_file, parse_rank)), |(file, rank)| (Some(file), Some(rank)))
	))(input)
}

fn parse_move<'a>(input: &'a str) -> IResult<&'a str, SANChessMove, VerboseError<&'a str>> {
	complete(alt((
		map(parse_castle_side, |side| SANChessMove::Castle(side)),
		parse_draw,
		map(parse_eog_state, |state| SANChessMove::EOG(state)),
		map(
			tuple((
				parse_piece,
				alt((
					map(parse_square, |square| ((None, None), false, square)),
					tuple((parse_disambiguator, parse_capture, parse_square))
				)),
				opt(parse_piece),
				parse_check_state,
			)),
			|(piece, ((src_file, src_rank), capture, dest), promotion, check)| SANChessMove::Move {
				piece,
				src_file,
				src_rank,
				capture,
				dest,
				promotion,
				check,
			},
		),
	)))(input)
}

impl<'a> TryFrom<&'a str> for SANChessMove {
	type Error = Err<VerboseError<&'a str>>;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		let (_, mv) = parse_move(value)?;
		Ok(mv)
	}
}

impl FromStr for SANChessMove {
	type Err = SANError;

	fn from_str<'a>(s: &'a str) -> Result<Self, Self::Err> {
		s.try_into().map_err(|e| SANError::Parse(match e {
			Err::Error(e) | Err::Failure(e) => convert_error(s, e),
			Err::Incomplete(_) => unreachable!(), // parse_move consumes the entire string, so an imcomplete parser is impossible
		}))
	}
}

impl From<SANChessMove> for String {
	fn from(mv: SANChessMove) -> Self {
		mv.to_string()
	}
}

fn piece_to_san(piece: &Piece) -> &str {
	match piece {
		Piece::Bishop => "B",
		Piece::Queen => "Q",
		Piece::Pawn => "",
		Piece::Rook => "R",
		Piece::King => "K",
		Piece::Knight => "N",
	}
}

impl Display for SANChessMove {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SANChessMove::Castle(side) => write!(f, "{}", side),
			SANChessMove::EOG(state) => write!(f, "{}", state),
			SANChessMove::OfferDraw => f.write_str("="),
			SANChessMove::Move {
				piece,
				capture,
				src_rank,
				src_file,
				dest,
				promotion,
				check,
			} => {
				f.write_str(piece_to_san(piece))?;

				if let Some(src_rank) = src_rank {
					f.write_str(&(src_rank.to_index() + 1).to_string())?;
				}

				if let Some(src_file) = src_file {
					f.write_str(match src_file {
						File::A => "a",
						File::B => "b",
						File::C => "c",
						File::D => "d",
						File::E => "e",
						File::F => "f",
						File::G => "g",
						File::H => "h",
					})?;
				}

				if *capture {
					f.write_str("x")?;
				}

				f.write_str(&dest.to_string())?;

				if let Some(promotion) = promotion {
					f.write_str(piece_to_san(promotion))?;
				}

				match check {
					CheckState::Check => f.write_str("+")?,
					CheckState::Double => f.write_str("++")?,
					CheckState::Mate => f.write_str("#")?,
					CheckState::None => {}
				}

				Ok(())
			}
		}
	}
}
