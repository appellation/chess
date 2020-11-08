FROM node

WORKDIR /usr/chess

COPY yarn.lock package.json .pnp.js .yarnrc.yml tsconfig.json ./
COPY .yarn .yarn
COPY services/bot/package.json services/bot/tsconfig.json services/bot/

RUN yarn install

COPY services/bot services/bot
WORKDIR /usr/chess/services/bot

RUN yarn build
CMD ["yarn", "start"]
