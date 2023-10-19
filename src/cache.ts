import { Opt, none, some } from "ts-opt";

export default class Cache<T> {
    lastUpdated: number;
    ttl: number;
    private _data: Opt<T> = none;
    private fetcher: () => Promise<T>;

    /**
     * Creates a new Cache instance
     * @param ttl The time the cache is valid in milliseconds. If you load the data after the ttl, the class will make a request using the fetcher.
     * @param fetcher The method used to fetch the data.
     */
    constructor(ttl: number, fetcher: () => Promise<T>) {
        this.ttl = ttl;
        this.fetcher = fetcher;
        this.lastUpdated = 0;
    }

    /**
     * Returns the last update date as a Date object.
     */
    public get lastUpdateDate(): Date {
        return new Date(this.lastUpdated);
    }

    /**
     * Returns the data from the cache or loads it from the fetcher if the cache is invalid (e.g. after the ttl expired).
     */
    public get data(): Promise<T> {
        return new Promise((resolve, reject) => {
            this._data.filter(c => this.lastUpdated > Date.now() - this.ttl).onBoth(
                // Cache-Hit
                (cachedData) => {
                    console.log("Cache-Hit");
                    resolve(cachedData);
                },

                // Cache-Miss
                async () => {
                    console.log("Cache-Miss")
                    try {
                        resolve(await this.loadData());
                    } catch (error) {
                        reject(error)
                    }
                }
            )
        });
    }

    /**
     * Refreshes the data stored in the cache.
     * @returns The data from the fetcher.
     */
    public async loadData(): Promise<T> {
        try {
            const data = await this.fetcher();
            this._data = some(data);
            this.lastUpdated = Date.now();
            return data;
        } catch (error) {
            throw error;
        }
    }
}