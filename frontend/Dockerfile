FROM node:22-alpine AS build

ARG URL
ENV PUBLIC_BACKEND_URL=${URL}
ENV PUBLIC_IS_APP=false

WORKDIR /app

COPY frontend/package.json package-lock.json frontend/postcss.config.js frontend/svelte.config.js frontend/tailwind.config.ts frontend/tsconfig.json frontend/vite.config.js ./
COPY frontend/src ./src
COPY frontend/static ./static

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