import express, { Request, Response, NextFunction } from 'express';
import { getLastUpdate, getMenu } from '../../v1/data.js';
import { newLocationEnum, parseDay } from '../../param_parsers.js';

const router = express.Router();


router.get('/', async (req: Request, res: Response, next: NextFunction) => {
    try {
        const params = req.query;
        let data = await getMenu(newLocationEnum(params.mensa?.toString()));

        if (params.date && (params.day || params.week)) {
            res.status(400).json({
                "error": "You can't use the date parameter in combination with day or week"
            });
            return;
        }

        if (params.date) {
            // Check if date is in format dd-mm-yyyy
            if (!params.date.toString().match(/^\d{2}-\d{2}-\d{4}$/)) {
                res.status(400).json({
                    "error": "Invalid date format"
                });
                return;
            }
            // Parse as dd-mm-yyyy
            const date = new Date(params.date.toString().split("-").reverse().join("-"));
            // Check if date is valid
            if (isNaN(date.getTime())) {
                res.status(400).json({
                    "error": "Invalid date"
                });
                return;
            }
            data = data.filter(day => day.date.getDate() === date.getDate() && day.date.getMonth() === date.getMonth() && day.date.getFullYear() === date.getFullYear());
        }
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