require('dotenv').config();
const express = require('express');
const cors = require('cors');
const { errorHandler, notFound } = require('./middleware.js');
const getSpeiseplan = require('./scraper');

const app = express();
app.use(cors());


const PORT = process.env.PORT || 3000;

const weekdays = ["so", "mo", "di", "mi", "do", "fr", "sa"];

app.get('/', (req, res) => {
    res.json({
        meals: '/meals',
    })
});

app.get('/meals', async (req, res, next) => {
    try {
        let data = await getSpeiseplan();

        const params = req.query;
        if (params.day) {
            data = data.filter(day => new Date(Date.parse(day.date)).getDay() === weekdays.indexOf(params.day.toLowerCase()));
        }
        if (params.week) {
            data = data.filter(day => day.week === parseInt(params.week));
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