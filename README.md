This project is a web scraper that searches the website of the Mensa of the University of Lübeck and extracts information about the menu. It allows access to the current menus and their details, such as names, prices and vegetarian options.

## API

### Endpoints

#### GET /meals

Returns a list of all meals.

##### Parameters

| Name    | Type   | Description                                                                                                                               |
| ------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `day`   | string | The day of the week. Valid values are `mon`, `tue`, `wed`, `thu`, `fri`, `sat` and `sun`.                                                                    |
| `week`  | string | The week. Valid values are `current` for the current week and `next` for the next                                                         |
| `mensa` | string | Location. Valid values are `mh` for the cafeteria in the 'Musikhochschule'. everything else defaults back to 'Mensa Lübeck mit Cafeteria' |

#### GET /allergens

Returns a list of all allergens.

#### GET /refresh

Refresh all caches.

#### GET /allergens/last-update

Returns the ISO Date of the last update of the allergens cache.

#### GET /meals/last-update

Returns the ISO Date of the last update of the meals cache.

##### Parameters

| Name    | Type   | Description                                                                                                                               |
| ------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `mensa` | string | Location. Valid values are `mh` for the cafeteria in the 'Musikhochschule'. everything else defaults back to 'Mensa Lübeck mit Cafeteria' |

##### Example

```bash
curl https://speiseplan.mcloud.digital/meals?day=fr | jq
```

## Configuration

The app is configured via environment variables. The following variables are available:

| Name              | Description                                                                                                                               |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `PORT`            | The port the app will listen on. Defaults to `3000`.                                                                                      |
| `CACHE_TTL_MENU`       | The time to live of the menu cache containing the meals in milliseconds. Defaults to `1000 * 60 * 10`.                                                                             |
| `CACHE_TTL_ALLERGENS` | The time to live of the allergens cache in milliseconds. Defaults to `1000 * 60 * 60 * 24`.                                                                             |

## Local Setup

### Requirements

- Node
- npm

### Installation

```bash
npm install
```

### Run

```bash
npm run dev
```

Per default the app will run on port 3000. You can change this by setting the `PORT` environment variable.

## Deployment

The app is deployed on a kubernetes cluster. To deploy a new version, just push to the master branch. The cluster will
automatically pull the latest version and restart the app.

The app is available at https://speiseplan.mcloud.digital .

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
