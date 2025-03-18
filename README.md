This project is a web scraper that scans the canteen website of the University of Lübeck and extracts information about the menu. It provides access to the current menus and their details, such as names, prices and vegetarian options.

## API

Since the first version of this scraper was written in Typescript and therefore used an exorbitant amount of memory when running, I rewrote it in Rust.  
I also took the opportunity to improve the design of the REST API. The documentation of the new API v2 is available [here](https://github.com/Draculente/mensa-api/blob/main/openapi.yaml).  

### Arrays

The API allows many query parameters to have multiple values (like arrays). Just separate the values with commas.

#### Example

```bash
curl https://speiseplan.mcloud.digital/v2/meals?location=HL_ME,HL_MH | jq
```

## Configuration

The app is configured via environment variables. The following variables are available:

| Name                  | Description                                                                                            |
| --------------------- | ------------------------------------------------------------------------------------------------------ |
| `PORT`                | The port the app will listen on. Defaults to `3030`.                                                   |
| `TTL`      | The time to live of the menu cache containing the meals in seconds. Defaults to `60 * 45`. |

## Local Setup

### Requirements

- Rust

### Run

```bash
cargo run
```

Per default the app will run on port 3030. You can change this by setting the `PORT` environment variable.

## Deployment

The app is deployed on a kubernetes cluster. To deploy a new version, just push to the main branch. The cluster will
automatically pull the latest version and restart the app.

The app is available at https://speiseplan.mcloud.digital/v2 .

## Gitmoji

This project uses [gitmoji](https://gitmoji.carloscuesta.me/) to make commits more expressive.

### Installation

```bash
npm install -g gitmoji-cli
```

### Initialize as git hook

```bash
gitmoji -i
```

## Used in 

- [Tray Application](https://github.com/Importantus/speiseplan-tray/) (Windows, Linux)
- [KDE Plasma Widget](https://github.com/lomenzel/mensa) (Linux with KDE Plasma)
- [Android Widget](https://github.com/hoppjan/LuebeckMensaWidget) (Android)
- [MS Teams Bot](https://github.com/budel/Mensa-Bot) (Microsoft Teams)

If you use the API in your project feel free to add it here with a PR or just open an issue.