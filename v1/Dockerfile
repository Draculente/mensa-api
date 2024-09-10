FROM node:latest

# Create app directory
WORKDIR /app

# Install app dependencies
COPY . /app

# Install dependencies
RUN npm ci

# Build app
RUN npm run build

CMD [ "node", "./dist" ]
