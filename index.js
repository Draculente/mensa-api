require('dotenv').config();
const express = require('express');
const cors = require('cors');
const { errorHandler, notFound } = require('./middleware.js');
const getSpeiseplan = require('./scraper');

const app = express();
app.use(cors());


const PORT = process.env.PORT || 3000;

const weekdays = ["mo", "di", "mi", "do", "fr", "sa", "so"];

app.get('/', (req, res) => {
    res.json({
        meals: '/meals',
    })
});

app.options('/meals', (req, res) => {
    res.send("get")
});

app.get('/meals', async (req, res, next) => {
    try {
        let data = await getSpeiseplan();

        const params = req.query;
        if (params.day) {
            data = data.filter(day => new Date(Date.parse(day.date)).getDay() === weekdays.indexOf(params.day.toLowerCase()));
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