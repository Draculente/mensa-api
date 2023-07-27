import fetch from "node-fetch";
import jsdom from "jsdom";
import he from "he";
import {Opt, none, opt} from "ts-opt";

interface Allergenes {
    code: string;
    name: string;
}

interface Meal {
    name: string;
    price: string;
    allergens: Allergenes[];
}

enum HasError {
    HAS_ERROR = "has_error",
    NO_ERROR = "no_error",
}

enum Week {
    CURRENT_WEEK = "current",
    NEXT_WEEK = "next",
}

interface Day {
    date: Date;
    week: Week;
    open: boolean;
    hasError: HasError;
    meals: Meal[];
}

type Speiseplan = Day[];

// Funktion zum Abrufen des Speiseplans
async function fetchSpeiseplan(ort: number): Promise<Speiseplan> {
    const result: Speiseplan = []; // Array zum Speichern der Ergebnisse

    const weeks = [0, 1]; // Array mit den Wochen, für die der Speiseplan abgerufen werden soll

    for (const week of weeks) {
        // URL für die aktuelle Woche generieren
        const url = `https://studentenwerk.sh/de/mensen-in-luebeck?ort=3&mensa=${ort == 0 ? 8 : 9}&nw=${week}#mensaplan`
        // HTTP-Anfrage an die URL senden und HTML-Daten abrufen
        const document = await fetch(url)
            .then((res) => res.text()) // Die Antwort in Text umwandeln
            .then((body) => new jsdom.JSDOM(body).window.document); // HTML-Dokument erstellen

        const dates = getWeekDates(week); // Array der nächsten 6 Termine (ohne Samstag und Sonntag)
        const allergens = getAllergens(document); // Array der Allergene

        // Schleife zum Extrahieren der Mahlzeiten für jeden Termin
        for (const date of dates) {
            const meals = getMealsByDate(document, date); // Element mit den Mahlzeiten für das aktuelle Datum auswählen
            const {open, meals: mealsArray, hasError} = extractMealInformation(meals, allergens); // Array mit den extrahierten Mahlzeiten

            // Die Ergebnisse für das aktuelle Datum zum Ergebnis-Array hinzufügen
            result.push({
                date,
                week: week === 0 ? Week.CURRENT_WEEK : Week.NEXT_WEEK,
                open: open ?? false,
                meals: mealsArray!,
                hasError: hasError ?? HasError.HAS_ERROR,
            });
        }
    }

    return result; // Das Ergebnis zurückgeben
}

function getWeekDates(offset = 0): Date[] {
    const dates = [];
    const now = new Date(); // Aktuelles Datum und Uhrzeit

    const realDayArray = [6, 0, 1, 2, 3, 4, 5];

    // Setze now auf den letzten Montag und füge den Offset hinzu
    now.setDate(now.getDate() - realDayArray[now.getDay()] + offset * 7);

    // Schleife zum Generieren der Termine für die nächsten 5 Werktage (Montag bis Freitag)
    for (let index = 0; index < 7; index++) {
        const date = new Date(now);
        date.setDate(date.getDate() + index);
        dates.push(date); // Das Datum zum Array hinzufügen
    }

    return dates;
}

// Funktion zum Extrahieren der Allergene aus dem Dokument
function getAllergens(document: Document): Allergenes[] {
    const allergeneParent = document.querySelector(".mbf_content");
    const allergeneElements: Opt<Element[]> = opt(allergeneParent?.children).map(htmlCollectionToArray);

    if (allergeneElements.isSome()) {
        return allergeneElements.value.map((allergeneElement) => ({
            code: allergeneElement.getAttribute("data-wert") ?? "",
            name: allergeneElement.children[1].innerHTML ?? "",
        }))
    }

    return [];
}

function htmlCollectionToArray(collection: HTMLCollection): Element[] {
    const array = [];
    for (let i = 0; i < collection.length; i++) {
        array.push(collection[i]);
    }
    return array;
}

// Funktion zum Extrahieren der Mahlzeiten für ein bestimmtes Datum
function getMealsByDate(document: Document, date: Date): Opt<Element> {
    const isoDate = date.toISOString().slice(0, 10);
    return opt(document.querySelector(`[data-day="${isoDate}"]:not(.mb_day)`));
}

// Funktion zum Extrahieren der Informationen für jede Mahlzeit
function extractMealInformation(meals: Opt<Element>, allergens: Allergenes[]): Partial<Day> {
    if (meals.isNone()) return {open: true, meals: [], hasError: HasError.HAS_ERROR}; // Wenn keine Mahlzeiten gefunden wurden, wird ein leeres Array zurückgegeben

    // Prüfen, ob die Mensa geschlossen ist
    const closed = meals.map((meals) => {
        const closedElement = meals.querySelector(".mensa_menu_geschlossen");
        return closedElement && closedElement.innerHTML.includes("geschlossen");
        ;
    }).orElse(false);

    // Wenn die Mensa geschlossen ist, wird ein leeres Array zurückgegeben
    if (closed) {
        return {open: false, meals: [], hasError: HasError.NO_ERROR};
    }

    const mealsArray: Opt<Meal[]> = meals.map((meals) => {
        const mealsInfos = htmlCollectionToArray(meals.getElementsByClassName("mensa_menu_detail")); // Alle Mahlzeiteninformationen auswählen

        return mealsInfos.map((mealInfo) => {
            let name = opt(mealInfo
                .querySelector(".menu_name")
                ?.innerHTML.split(
                    /<\/?\w+((\s+\w+(\s*=\s*(?:".*?"|'.*?'|[\^'">\s]+))?)+\s*|\s*)\/?>/
                ).filter((item) => item && !item.startsWith("(") && !item.includes("="))
                .map((item) => item.trim())
                .join(", ")
                .replaceAll(/(\W)\1+/g, "$1"))
                .map(he.decode)
                .orElse("Error getting name"); // Den Namen der Mahlzeit extrahieren und HTML-Tags entfernen

            const price = opt(mealInfo.querySelector(".menu_preis")?.textContent).map((price) => price.trim()).orElse("Error getting price"); // Den Preis der Mahlzeit extrahieren
            const vegan = mealInfo.getAttribute("data-arten")?.includes("vn") ?? false; // Überprüfen, ob die Mahlzeit vegan ist
            const vegetarian = (mealInfo.getAttribute("data-arten")?.includes("ve") ?? false) || vegan; // Überprüfen, ob die Mahlzeit vegetarisch ist
            const location = mealInfo.querySelector(".menu_art")?.textContent ?? "Error getting location"; // Den Standort der Mahlzeit extrahieren

            const rawAllergens = mealInfo
                .querySelector(".menu_name")
                ?.textContent ?? ""; // Alle Allergene extrahieren

            const mealAllergens = allergens.filter((allergene) => rawAllergens.includes(allergene.code)); // Die Allergene der Mahlzeit extrahieren

            return {
                name,
                price,
                vegan,
                vegetarian,
                location,
                allergens: mealAllergens,
            }

        });

    }); // Die Mahlzeiteninformationen extrahieren

    return {
        open: true,
        meals: mealsArray.orElse([]),
        hasError: mealsArray.map(() => HasError.NO_ERROR).orElse(HasError.HAS_ERROR)
    }; // Das Array mit den Mahlzeiten zurückgeben
}

const CACHE_TTL: number = +(process.env.CACHE_TTL || 1000 * 60 * 60 * 4);

interface Cache {
    data: Speiseplan;
    lastUpdated: number;
}

let cache_th: Opt<Cache> = none
let cache_mh: Opt<Cache> = none

export function getSpeiseplan(ort: number): Promise<Speiseplan> {
    if (ort == 0)
        return new Promise((resolve, reject) => {
            cache_th.filter(c => c.lastUpdated > Date.now() - CACHE_TTL).onBoth(
                // Cache-Hit
                (cache) => {
                    resolve(cache.data);
                },
                // Cache-Miss
                async () => {
                    const data = await fetchSpeiseplan(0);
                    cache_th = opt({
                        data,
                        lastUpdated: Date.now(),
                    });
                    resolve(data);
                }
            );
        });
    else
        return new Promise((resolve, reject) => {
            cache_mh.filter(c => c.lastUpdated > Date.now() - CACHE_TTL).onBoth(
                // Cache-Hit
                (cache) => {
                    resolve(cache.data);
                },
                // Cache-Miss
                async () => {
                    const data = await fetchSpeiseplan(1);
                    cache_mh = opt({
                        data,
                        lastUpdated: Date.now(),
                    });
                    resolve(data);
                }
            );
        });

}