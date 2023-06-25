import express, { Request, Response, NextFunction } from 'express';
import cors from 'cors';
import { errorHandler, notFound } from './middleware.js';
import { getSpeiseplan } from './scraper.js';

const app = express();
app.use(cors());


const PORT = process.env.PORT || 3000;

const weekdays = ["so", "mo", "di", "mi", "do", "fr", "sa"];

app.get('/', (req: Request, res: Response) => {
    res.json({
        meals: '/meals',
    })
});

app.get('/meals', async (req: Request, res: Response, next: NextFunction) => {
    try {
        let data = await getSpeiseplan();

        const params = req.query;
        if (params.day) {
            data = data.filter(day => new Date(day.date).getDay() === weekdays.indexOf(params.day?.toString().toLowerCase() ?? ""));
        }
        if (params.week) {
            data = data.filter(day => day.week === params.week?.toString());
        }

        res.json(data);
    } catch (error) {
        next(error);
    }
});

app.use(notFound);
app.use(errorHandler);

app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
});