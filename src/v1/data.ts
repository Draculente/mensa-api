import Cache from "../cache.js";
import { LocationEnum } from "../param_parsers.js";
import { Allergen, Menu, fetchAllergens, fetchSpeiseplan } from "../scraper.js";

const CACHE_TTL_MENU: number = +(process.env.CACHE_TTL_MENU || 1000 * 60 * 10);
const CACHE_TTL_ALLERGENS: number = +(process.env.CACHE_TTL_ALLERGENS || 1000 * 60 * 60 * 24);

interface Location {
    allergens: Cache<Allergen[]>;
    menu: Cache<Menu>;
}

interface CacheStore {
    [LocationEnum.TH]: Location;
    [LocationEnum.MH]: Location;
}

let store: CacheStore = {
    [LocationEnum.TH]: {
        allergens: new Cache(CACHE_TTL_ALLERGENS, () => fetchAllergens(LocationEnum.TH)),
        menu: new Cache(CACHE_TTL_MENU, () => fetchSpeiseplan(LocationEnum.TH)),
    },
    [LocationEnum.MH]: {
        allergens: new Cache(CACHE_TTL_ALLERGENS, () => fetchAllergens(LocationEnum.MH)),
        menu: new Cache(CACHE_TTL_MENU, () => fetchSpeiseplan(LocationEnum.MH)),
    }
};

/**
 * Returns the menu. The menu is scraped from the Studentenwerk website and cached.
 * @param location The location the menu is scraped for.
 * @returns A promise that resolves to an array of days.
 */
export function getMenu(location: LocationEnum): Promise<Menu> {
    return store[location].menu.data;
}

/**
 * Returns all allergens. The allergens are scraped from the Studentenwerk website and cached.
 * @param location The location the allergens are scraped for.
 * @returns A promise that resolves to an array of allergens.
 */
export function getAllergens(location: LocationEnum): Promise<Allergen[]> {
    return store[location].allergens.data;
}

/**
 * Refreshes all caches.
 * @returns A promise that resolves when all caches are refreshed.
 */
export async function refresh() {
    return Promise.all(
        Object.values(store).map(location => {
            return Promise.all([
                location.allergens.refresh(),
                location.menu.refresh()
            ]);
        })
    )
}

/**
 * Returns the last update date of a cache.
 * @param location The location the cache stores.
 * @param menu If the cache stores the menu or the allergens.
 * @returns The last update date of the cache.
 */
export function getLastUpdate(location: LocationEnum, menu: boolean): Date {
    return menu ? store[location].menu.lastUpdateDate : store[location].allergens.lastUpdateDate;
}