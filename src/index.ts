import express from 'express';
import cors from 'cors';
import { errorHandler, notFound } from './middleware.js';
import mealsRouterV1 from './routers/v1/meals.js';
import allergensRouterV1 from './routers/v1/allergens.js';
import refreshRouterV1 from './routers/v1/refresh.js';

const app = express();
app.use(cors());

interface APIVersion {
    path: string;
    name: string;
    resources: Resource[];
}

interface Resource {
    path: string;
    router?: express.Router;
    name: string;
}

const versions: APIVersion[] = [
    {
        path: "",
        name: "v1",
        resources: [
            {
                path: "/meals",
                name: "meals",
                router: mealsRouterV1
            },
            {
                path: "/allergens",
                name: "allergens",
                router: allergensRouterV1
            },
            {
                path: "/refresh",
                name: "refresh",
                router: refreshRouterV1
            }
        ]
    },
];


app.get("/", (_, res) => {
    res.json(versions.map(version => {
        return {
            version: version.name,
            path: `/${version.path}`
        }
    }));
});

versions.forEach(version => {
    app.get(`${version.path}`, (_, res) => {
        res.json(version.resources.map(resource => {
            return {
                path: `${version.path}${resource.path}`,
                name: resource.name
            }
        }));
    });
})

versions.forEach(version => {
    version.resources.forEach(resource => {
        if (resource.router) {
            console.log(`Registering ${resource.name} at ${version.path}${resource.path}`);

            app.use(`${version.path}${resource.path}`, resource.router);
        }
    });
});


app.use(notFound);
app.use(errorHandler);

const PORT = process.env.PORT || 3000;

app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
});