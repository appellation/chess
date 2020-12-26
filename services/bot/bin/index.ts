import { Amqp, AmqpResponseOptions } from '@spectacles/brokers';
import Client from '@spectacles/proxy';
import { User as DiscordUser, Interaction, InteractionType } from '@spectacles/types';
import fetch from 'node-fetch';

const apiUrl = process.env.API_URL ?? 'http://localhost:8080';
const boardsUrl = process.env.BOARDS_URL ?? 'http://localhost:8081';
const amqpUrl = process.env.AMQP_URL ?? 'localhost';
const token = process.env.DISCORD_TOKEN;

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

function respondToGame(interaction: Interaction, game: Game) {
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

	respond(interaction, content);
}

function respond(interaction: Interaction, content: string) {
	proxy.post(`/interactions/${interaction.id}/${interaction.token}/callback`, { type: 4, data: { content, allowed_mentions: { parse: ['users'] } } }).catch(console.error);
}

broker.on('INTERACTION_CREATE', async (interaction: Interaction, { ack }: AmqpResponseOptions) => {
	ack();

	console.log(require('util').inspect(interaction, { depth: Infinity }));

	switch (interaction.type) {
		case InteractionType.PING: {
			proxy.post(`/interactions/${interaction.id}/${interaction.token}/callback`, { type: 1 }).catch(console.error);
			break;
		}
		case InteractionType.APPLICATION_COMMAND: {
			switch (interaction.data?.name) {
				case 'ping': {
					respond(interaction, 'pong');
					break;
				}
				case 'challenge': {
					const res = await fetch(`${apiUrl}/games`, {
						method: 'post',
						headers: {
							'x-user-id': interaction.member.user.id,
							'x-account-type': 'Discord',
						},
						body: JSON.stringify({
							target_id: interaction.data?.options[0].value,
							account_type: 'Discord',
						}),
					});

					if (!res.ok) {
						respond(interaction, 'can\'t create game');
						return;
					}

					const game = await res.json();
					respondToGame(interaction, game);
					break;
				}
				case 'game': {
					const res = await fetch(`${apiUrl}/games/current`, {
						method: 'get',
						headers: {
							'x-user-id': interaction.member.user.id,
							'x-account-type': 'Discord',
						},
					});

					if (!res.ok) {
						respond(interaction, 'no game');
						return;
					}

					const game = await res.json();
					respondToGame(interaction, game);

					break;
				}
				case 'move': {
					const res = await fetch(`${apiUrl}/games/current/moves`, {
						method: 'put',
						headers: {
							'x-user-id': interaction.member.user.id,
							'x-account-type': 'Discord',
						},
						body: JSON.stringify({
							action: 'MakeMove',
							data: interaction.data?.options[0].value,
						}),
					});

					if (!res.ok) {
						respond(interaction, 'invalid move');
						return;
					}

					const game = await res.json();
					respondToGame(interaction, game);
					break;
				}
				case 'resign': {
					const res = await fetch(`${apiUrl}/games/current/moves`, {
						method: 'put',
						headers: {
							'x-user-id': interaction.member.user.id,
							'x-account-type': 'Discord',
						},
						body: JSON.stringify({
							action: 'Resign',
						}),
					});

					if (!res.ok) {
						respond(interaction, 'invalid move');
						return;
					}

					const game = await res.json();
					respondToGame(interaction, game);
					break;
				}
				case 'pgn': {
					const res = await fetch(`${apiUrl}/games/previous`, {
						method: 'get',
						headers: {
							'x-user-id': interaction.member.user.id,
							'x-account-type': 'Discord',
						},
					});

					if (!res.ok) {
						respond(interaction, 'unable to get last game');
						return;
					}

					const game = await res.json();
					respond(interaction, `\`\`\`\n${game.pgn}\n\`\`\``);
					break;
				}
				case 'help': {
					respond(interaction, 'check out my available slash (/) commands!');
					break;
				}
			}
		}
	}
});

broker.on('close', console.log);

(async () => {
	try {
		console.log('connecting....');
		const connection = await broker.connect(amqpUrl);
		broker.subscribe('INTERACTION_CREATE');
		await restBroker.connect(connection);
		console.log('ready');
	} catch (e) {
		console.error(e);
		process.exit(1);
	}
})();
