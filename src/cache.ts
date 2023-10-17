import { Opt, none, some } from "ts-opt";

export default class Cache<T> {
    lastUpdated: number;
    ttl: number;
    private _data: Opt<T> = none;
    private fetcher: () => Promise<T>;

    constructor(ttl: number, fetcher: () => Promise<T>) {
        this.ttl = ttl;
        this.fetcher = fetcher;
        this.lastUpdated = 0;
    }

    public get lastUpdateDate(): Date {
        return new Date(this.lastUpdated);
    }

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
                        resolve(await this.refresh());
                    } catch (error) {
                        reject(error)
                    }
                }
            )
        });
    }

    public async refresh(): Promise<T> {
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