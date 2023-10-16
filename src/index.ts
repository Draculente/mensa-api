import express, { Request, Response, NextFunction } from 'express';
import cors from 'cors';
import { errorHandler, notFound } from './middleware.js';
import { Ort, getMensaData } from './scraper.js';

const app = express();
app.use(cors());


const PORT = process.env.PORT || 3000;

const weekdays = [
    ["so", "mo", "di", "mi", "do", "fr", "sa"],
    ["su", "mo", "tu", "we", "th", "fr", "sa"],
    ["sun", "mon", "tue", "wed", "thu", "fri", "sat"],
    ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"]
]

app.get('/', (req: Request, res: Response) => {
    res.json({
        meals: '/meals',
        allergens: '/allergens'
    })
});

app.get('/meals', async (req: Request, res: Response, next: NextFunction) => {
    try {
        let data = null;
        const params = req.query;
        data = await getMensaData(params.mensa === "mh" ? Ort.MH : Ort.TH);
        data = data.speiseplan

        if (params.day) {
            data = data.filter(day => new Date(day.date).getDay() === weekdays.find(weekday => weekday.includes(params.day?.toString().toLowerCase() ?? ""))?.indexOf(params.day?.toString().toLowerCase() ?? ""));
        }
        if (params.week) {
            data = data.filter(day => day.week === params.week?.toString());
        }

        res.json(data);
    } catch (error) {
        next(error);
    }
});

app.get('/allergens', async (req: Request, res: Response, next: NextFunction) => {
    try {
        let data = null;
        const params = req.query;
        data = await getMensaData(params.mensa === "mh" ? Ort.MH : Ort.TH);
        data = data.allergens

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