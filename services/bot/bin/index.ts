import { Amqp, AmqpResponseOptions } from '@spectacles/brokers';
import Client from '@spectacles/proxy';
import { Message } from '@spectacles/types';
import { Lexer, Parser, Args } from 'lexure';
import fetch from 'node-fetch';

const apiUrl = process.env.API_URL ?? 'http://localhost:8080';
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

broker.on('MESSAGE_CREATE', async (message: Message, { ack }: AmqpResponseOptions) => {
	ack();

	const lexer = new Lexer(message.content);
	lexer.setQuotes([
		['"', '"']
	]);

	const output = lexer.lexCommand(s => s.startsWith('.') ? 1 : null);
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

			const gameId = await res.text();
			console.log(gameId);
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
