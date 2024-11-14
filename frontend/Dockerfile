FROM node:22-alpine AS build

ARG URL
ENV PUBLIC_BACKEND_URL=${URL}

WORKDIR /app

COPY package.json package-lock.json postcss.config.js svelte.config.js tailwind.config.ts tsconfig.json vite.config.js ./
COPY src ./src
COPY static ./static

RUN rm src/routes/+layout.ts
RUN sed -i 's/static/node/g' svelte.config.js

RUN npm i

RUN npm run build

FROM node:22-alpine

RUN adduser -D static
USER static
WORKDIR /app

COPY --from=build /app/build .
COPY --from=build /app/package.json .
COPY --from=build /app/package-lock.json .

CMD ["node", "."]