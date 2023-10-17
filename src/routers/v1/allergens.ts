import express, { Request, Response, NextFunction } from 'express';
import { getAllergens, getLastUpdate } from '../../scraper.js';
import { newLocationEnum } from '../../param_parsers.js';

const router = express.Router();

router.get('/', async (req: Request, res: Response, next: NextFunction) => {
    try {
        const params = req.query;
        let data = await getAllergens(newLocationEnum(params.mensa?.toString()));

        res.json(data);
    } catch (error) {
        next(error);
    }
});

router.get("/last-update", async (req: Request, res: Response, next: NextFunction) => {
    const params = req.query;
    res.json({
        "lastUpdate": getLastUpdate(newLocationEnum(params.mensa?.toString()), false)
    });
});


export default router;