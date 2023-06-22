const fetch = require("node-fetch");
const jsdom = require("jsdom");

// Die URL des Speiseplans
const url = "https://studentenwerk.sh/de/mensen-in-luebeck?ort=3&mensa=8#mensaplan";

// Funktion zum Abrufen des Speiseplans
async function fetchSpeiseplan() {
    // HTTP-Anfrage an die URL senden und HTML-Daten abrufen
    const document = await fetch(url)
        .then((res) => res.text()) // Die Antwort in Text umwandeln
        .then((body) => new jsdom.JSDOM(body).window.document); // HTML-Dokument erstellen

    const result = []; // Array zum Speichern der Ergebnisse

    const dates = []; // Array zum Speichern der Termine
    var dayOfWeek = [7, 1, 2, 3, 4, 5, 6]; // Array der Wochentage (0 = Sonntag, 1 = Montag, usw.)

    const now = new Date(); // Aktuelles Datum und Uhrzeit

    // Schleife zum Generieren der Termine für die nächsten 6 Tage (ohne Samstag und Sonntag)
    for (let index = 0; index < 7 - dayOfWeek[now.getDay()]; index++) {
        const date = now.setDate(new Date().getDate() + index); // Datum für den aktuellen Tag
        const day = dayOfWeek[now.getDay()]; // Wochentag für das aktuelle Datum
        if (day < 6) {
            dates.push(new Date(date)); // Das Datum zum Array hinzufügen, wenn es kein Samstag oder Sonntag ist
        }
    }

    const allergens = [];

    let allergeneElements = document.querySelector(".mbf_content").children

    for (let i = 0; i < allergeneElements.length; i++) {
        const allergene = {
            code: allergeneElements[i].attributes["data-wert"].value,
            name: allergeneElements[i].children[1].innerHTML
        }

        allergens.push(allergene)
    }

    // Schleife zum Extrahieren der Mahlzeiten für jeden Termin
    for (const day in dates) {
        const meals = document.querySelector(
            `[data-day="${dates[day].toISOString().slice(0, 10)}"]:not(.mb_day)`
        ); // Element mit den Mahlzeiten für das aktuelle Datum auswählen

        const mealsInfos = meals.getElementsByClassName("mensa_menu_detail"); // Alle Mahlzeiteninformationen auswählen

        let mealsArray = []; // Array zum Speichern der einzelnen Mahlzeiten

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
                .join(", "); // Den Namen bereinigen und formatieren

            const price = mealsInfos[i].querySelector(".menu_preis").textContent; // Den Preis der Mahlzeit extrahieren
            const vegetarian =
                mealsInfos[i].attributes["data-arten"].value.includes("ve") ||
                mealsInfos[i].attributes["data-arten"].value.includes("vn"); // Überprüfen, ob die Mahlzeit vegetarisch ist
            const vegan = mealsInfos[i].attributes["data-arten"].value.includes("vn");
            const location = mealsInfos[i].querySelector(".menu_art").textContent; // Den Ort der Mahlzeit extrahieren

            const mealAllergens = []

            for (let x = 0; x < allergens.length; x++) {
                if (mealsInfos[i]
                    .querySelector(".menu_name").textContent.includes(allergens[x].code)) {
                    mealAllergens.push(allergens[x])
                }
            }

            // Die extrahierten Informationen zur Mahlzeit zum Array hinzufügen
            mealsArray.push({
                name: name,
                price: price,
                vegetarian: vegetarian,
                vegan: vegan,
                location: location,
                allergens: mealAllergens
            });
        }

        // Die Ergebnisse für das aktuelle Datum zum Ergebnis-Array hinzufügen
        result.push({
            date: dates[day],
            meals: mealsArray,
        });
    }

    return result; // Das Ergebnis zurückgeben
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