const fetch = require("node-fetch");
const jsdom = require("jsdom");
const he = require("he");

// Funktion zum Abrufen des Speiseplans
async function fetchSpeiseplan() {
    const result = []; // Array zum Speichern der Ergebnisse

    const weeks = [0, 1]; // Array mit den Wochen, für die der Speiseplan abgerufen werden soll

    for (const week of weeks) {
        // URL für die aktuelle Woche generieren
        const url = `https://studentenwerk.sh/de/mensen-in-luebeck?ort=3&mensa=8&nw=${week}#mensaplan`
        // HTTP-Anfrage an die URL senden und HTML-Daten abrufen
        const document = await fetch(url)
            .then((res) => res.text()) // Die Antwort in Text umwandeln
            .then((body) => new jsdom.JSDOM(body).window.document); // HTML-Dokument erstellen

        const dates = getWeekDates(week); // Array der nächsten 6 Termine (ohne Samstag und Sonntag)
        const allergens = getAllergens(document); // Array der Allergene

        // Schleife zum Extrahieren der Mahlzeiten für jeden Termin
        for (const date of dates) {
            const meals = getMealsByDate(document, date); // Element mit den Mahlzeiten für das aktuelle Datum auswählen
            const { open, meals: mealsArray } = extractMealInformation(meals, allergens); // Array mit den extrahierten Mahlzeiten

            // Die Ergebnisse für das aktuelle Datum zum Ergebnis-Array hinzufügen
            result.push({
                date,
                week,
                open,
                meals: mealsArray,
            });
        }
    }

    return result; // Das Ergebnis zurückgeben
}

function getWeekDates(offset = 0) {
    const dates = [];
    const now = new Date(); // Aktuelles Datum und Uhrzeit

    // Setze now auf den letzten Montag und füge den Offset hinzu
    now.setDate(now.getDate() - now.getDay() + 1 + offset * 7);

    // Schleife zum Generieren der Termine für die nächsten 5 Werktage (Montag bis Freitag)
    for (let index = 0; index < 5; index++) {
        const date = new Date(now);
        date.setDate(date.getDate() + index);
        dates.push(date); // Das Datum zum Array hinzufügen
    }

    return dates;
}

// Funktion zum Extrahieren der Allergene aus dem Dokument
function getAllergens(document) {
    const allergens = [];
    const allergeneParent = document.querySelector(".mbf_content");
    const allergeneElements = allergeneParent ? allergeneParent.children : [];

    for (let i = 0; i < allergeneElements.length; i++) {
        const allergene = {
            code: allergeneElements[i].attributes["data-wert"].value,
            name: allergeneElements[i].children[1].innerHTML,
        };

        allergens.push(allergene);
    }

    return allergens;
}

// Funktion zum Extrahieren der Mahlzeiten für ein bestimmtes Datum
function getMealsByDate(document, date) {
    const isoDate = date.toISOString().slice(0, 10);
    const meals = document.querySelector(`[data-day="${isoDate}"]:not(.mb_day)`);

    return meals;
}

// Funktion zum Extrahieren der Informationen für jede Mahlzeit
function extractMealInformation(meals, allergens) {
    if (!meals) return { open: true, meals: [] }; // Wenn keine Mahlzeiten gefunden wurden, wird ein leeres Array zurückgegeben

    const dayIsClosed = meals.querySelector(".mensa_menu_geschlossen"); // Überprüfen, ob die Mensa an diesem Tag geschlossen ist
    if (dayIsClosed) return { open: false, meals: [] }; // Wenn die Mensa geschlossen ist, wird ein leeres Array zurückgegeben

    const mealsInfos = meals.getElementsByClassName("mensa_menu_detail"); // Alle Mahlzeiteninformationen auswählen
    const mealsArray = []; // Array zum Speichern der einzelnen Mahlzeiten

    // Schleife zum Extrahieren der Informationen für jede Mahlzeit
    for (let i = 0; i < mealsInfos.length; i++) {
        let name = mealsInfos[i]
            .querySelector(".menu_name")
            .innerHTML.split(
                /<\/?\w+((\s+\w+(\s*=\s*(?:".*?"|'.*?'|[\^'">\s]+))?)+\s*|\s*)\/?>/
            ); // Den Namen der Mahlzeit extrahieren und HTML-Tags entfernen

        name = name
            .filter((item) => item && !item.startsWith("(") && !item.includes("="))
            .map((item) => item.trim())
            .join(", ")
            .replaceAll(/(\W)\1+/g, "$1"); // Den Namen bereinigen und formatieren

        name = he.decode(name); // &amp; etc umwandeln

        const price = mealsInfos[i].querySelector(".menu_preis").textContent; // Den Preis der Mahlzeit extrahieren
        const vegetarian =
            mealsInfos[i].attributes["data-arten"].value.includes("ve") ||
            mealsInfos[i].attributes["data-arten"].value.includes("vn"); // Überprüfen, ob die Mahlzeit vegetarisch ist
        const vegan = mealsInfos[i].attributes["data-arten"].value.includes("vn");
        const location = mealsInfos[i].querySelector(".menu_art").textContent; // Den Ort der Mahlzeit extrahieren

        const mealAllergens = [];

        for (let x = 0; x < allergens.length; x++) {
            if (
                mealsInfos[i]
                    .querySelector(".menu_name")
                    .textContent.includes(allergens[x].code)
            ) {
                mealAllergens.push(allergens[x]);
            }
        }

        // Die extrahierten Informationen zur Mahlzeit zum Array hinzufügen
        mealsArray.push({
            name: name,
            price: price,
            vegetarian: vegetarian,
            vegan: vegan,
            location: location,
            allergens: mealAllergens,
        });
    }

    return { open: true, meals: mealsArray };
}

const CACHE_TTL = process.env.CACHE_TTL || 1000 * 60 * 60 * 4;

let cache = {
    data: null,
    lastUpdated: null
}

function getSpeiseplan() {
    return new Promise((resolve, reject) => {
        if (cache.data && cache.lastUpdated && cache.lastUpdated > Date.now() - CACHE_TTL) {
            resolve(cache.data);
        } else {
            return fetchSpeiseplan().then(data => {
                cache.data = data;
                cache.lastUpdated = Date.now();
                resolve(data);
            });
        }
    });
}

module.exports = getSpeiseplan;