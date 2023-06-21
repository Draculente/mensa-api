FROM node:latest

# Create app directory
WORKDIR /app

# Install app dependencies
COPY . /app

# Install dependencies
RUN npm ci

CMD [ "node", "." ]
