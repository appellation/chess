import { Amqp, AmqpResponseOptions } from '@spectacles/brokers';
import Client from '@spectacles/proxy';
import { Message } from '@spectacles/types';

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

broker.on('MESSAGE_CREATE', (message: Message, { ack }: AmqpResponseOptions) => {
	ack();

	if (message.content === 'ping') {
		proxy.post(`/channels/${message.channel_id}/messages`, { content: 'pong' }).catch(console.error);
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
