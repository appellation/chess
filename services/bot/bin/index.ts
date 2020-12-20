import { Amqp, AmqpResponseOptions } from '@spectacles/brokers';
import Client from '@spectacles/proxy';
import { Message } from '@spectacles/types';
import { Lexer, Parser, Args } from 'lexure';
import fetch from 'node-fetch';

const apiUrl = process.env.API_URL ?? 'http://localhost:8080';
const boardsUrl = process.env.BOARDS_URL ?? 'http://localhost:8081';
const amqpUrl = process.env.AMQP_URL ?? 'localhost';
const token = process.env.DISCORD_TOKEN;
const prefix = process.env.DISCORD_COMMAND_PREFIX ?? '.';

if (!token) throw new Error('missing DISCORD_TOKEN');

const broker = new Amqp('gateway', {
	reconnectTimeout: 5000,
});
const restBroker = new Amqp('rest', {
	reconnectTimeout: 5000,
});
const proxy = new Client(restBroker as any, token);

interface UserAccount {
	user_id: string;
	account_id: string;
	account_type: string;
}

interface User {
	id: string;
	accounts: UserAccount[];
}

interface Game {
	id: string;
	white: User;
	black: User;
	board: string;
	side_to_move: 'White' | 'Black';
	moves: string[];
	result: string | null;
}

function respondToGame(message: Message, game: Game) {
	const userToMove = game[game.side_to_move.toLowerCase() as 'white' | 'black'].accounts.find(account => account.account_type === 'Discord')?.account_id;
	const sideToMove = game.side_to_move.toLowerCase();

	let winnerId: string | undefined;
	let content: string;
	switch (game.result) {
		case 'BlackCheckmates':
		case 'WhiteResigns':
			winnerId = game.black.accounts.find(account => account.account_type === 'Discord')?.account_id;
			content = `<@${winnerId}> (black) wins!`;
			break;
		case 'WhiteCheckmates':
		case 'BlackResigns':
			winnerId = game.white.accounts.find(account => account.account_type === 'Discord')?.account_id;
			content = `<@${winnerId}> (white) wins!`;
			break;
		case 'Stalemate':
			content = 'Stalemate!';
			break;
		case 'DrawAccepted':
			content = 'Draw accepted.';
			break;
		case 'DrawDeclared':
			content = 'Draw declared.';
			break;
		default:
			content = `<@${userToMove}> (${sideToMove}) to move ${encodeURI(`${boardsUrl}/${game.board}`)}`;
			break;
	}

	proxy.post(`/channels/${message.channel_id}/messages`, { content }).catch(console.error);
}

broker.on('MESSAGE_CREATE', async (message: Message, { ack }: AmqpResponseOptions) => {
	ack();

	const lexer = new Lexer(message.content);
	lexer.setQuotes([
		['"', '"']
	]);

	const output = lexer.lexCommand(s => s.startsWith(prefix) ? 1 : null);
	if (!output) return;
	const [cmd, getTokens] = output;

	const parser = new Parser(getTokens());
	const args = new Args(parser.parse());

	console.log(cmd, args);
	switch (cmd.value) {
		case 'ping': {
			proxy.post(`/channels/${message.channel_id}/messages`, { content: 'pong' }).catch(console.error);
			break;
		}
		case 'challenge': {
			const res = await fetch(`${apiUrl}/games`, {
				method: 'post',
				headers: {
					'x-user-id': message.author.id,
					'x-account-type': 'Discord',
				},
				body: JSON.stringify({
					target_id: args.single(),
					account_type: 'Discord',
				}),
			});

			if (!res.ok) {
				proxy.post(`/channels/${message.channel_id}/messages`, { content: 'can\'t create game' }).catch(console.error);
				return;
			}

			const game = await res.json();
			respondToGame(message, game);
			break;
		}
		case 'game': {
			const res = await fetch(`${apiUrl}/games/current`, {
				method: 'get',
				headers: {
					'x-user-id': message.author.id,
					'x-account-type': 'Discord',
				},
			});

			if (!res.ok) {
				proxy.post(`/channels/${message.channel_id}/messages`, { content: 'no game' }).catch(console.error);
				return;
			}

			const game = await res.json();
			respondToGame(message, game);

			break;
		}
		case 'move': {
			const move = args.single();
			const res = await fetch(`${apiUrl}/games/current/moves`, {
				method: 'put',
				headers: {
					'x-user-id': message.author.id,
					'x-account-type': 'Discord',
				},
				body: JSON.stringify({
					action: 'MakeMove',
					data: move,
				}),
			});

			if (!res.ok) {
				proxy.post(`/channels/${message.channel_id}/messages`, { content: 'invalid move' }).catch(console.error);
				return;
			}

			const game = await res.json();
			respondToGame(message, game);
			break;
		}
		case 'resign': {
			const res = await fetch(`${apiUrl}/games/current/moves`, {
				method: 'put',
				headers: {
					'x-user-id': message.author.id,
					'x-account-type': 'Discord',
				},
				body: JSON.stringify({
					action: 'Resign',
				}),
			});

			if (!res.ok) {
				proxy.post(`/channels/${message.channel_id}/messages`, { content: 'invalid move' }).catch(console.error);
				return;
			}

			const game = await res.json();
			respondToGame(message, game);
			break;
		}
	}
});

broker.on('close', console.log);

(async () => {
	try {
		console.log('connecting....');
		const connection = await broker.connect(amqpUrl);
		broker.subscribe('MESSAGE_CREATE');
		await restBroker.connect(connection);
		console.log('ready');
	} catch (e) {
		console.error(e);
		process.exit(1);
	}
})();
