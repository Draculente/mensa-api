Dieses Projekt ist ein Web-Scraper, der die Website der Mensa der Hochschule Lübeck durchsucht und Informationen zum Speiseplan extrahiert. Es ermöglicht den Zugriff auf die aktuellen Menüs und deren Details, wie z.B. Namen, Preise und vegetarische Optionen.


## API

### Endpoints

#### GET /meals

Returns a list of all meals.

##### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| `day` | string | The day of the week. Valid values are 'mo', 'di', 'mi', 'do', 'fr', 'sa' and 'so'. Only days after the current will return a value. |

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
node .
```

Per default the app will run on port 3000. You can change this by setting the `PORT` environment variable.

## Deployment

The app is deployed on a kubernetes cluster. To deploy a new version, just push to the master branch. The cluster will automatically pull the latest version and restart the app.  

The app is available at https://speiseplan.mcloud.digital .
