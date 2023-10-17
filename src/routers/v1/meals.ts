import express, { Request, Response, NextFunction } from 'express';
import { getLastUpdate, getMenu } from '../../scraper.js';
import { newLocationEnum, parseDay } from '../../param_parsers.js';

const router = express.Router();


router.get('/', async (req: Request, res: Response, next: NextFunction) => {
    try {
        const params = req.query;
        let data = await getMenu(newLocationEnum(params.mensa?.toString()));

        if (params.day) {
            data = data.filter(day => new Date(day.date).getDay() === parseDay(params.day?.toString()));
        }
        if (params.week) {
            data = data.filter(day => day.week === params.week?.toString());
        }

        res.json(data);
    } catch (error) {
        next(error);
    }
});

router.get("/last-update", async (req: Request, res: Response, next: NextFunction) => {
    const params = req.query;
    res.json({
        "lastUpdate": getLastUpdate(newLocationEnum(params.mensa?.toString()), true)
    });
});

export default router;