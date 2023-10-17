import express, { Request, Response, NextFunction } from 'express';
import { refresh } from '../../scraper.js';

const router = express.Router();

router.get('/', async (req: Request, res: Response, next: NextFunction) => {
    try {
        await refresh();
        res.json({ "status": "refreshed" });
    } catch (error) {
        next(error);
    }
});

export default router;