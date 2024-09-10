import { Request, Response, NextFunction } from "express";

export const errorHandler = (error: any, req: Request, res: Response, next: NextFunction) => {
    const statusCode = res.statusCode === 200 ? 500 : res.statusCode;
    res.status(statusCode);
    res.json({
        message: error.message,
        stack: process.env.STATUS === "Production" ? "" : error.stack,
    });
};

export const notFound = (req: Request, res: Response, next: NextFunction) => {
    const error = new Error(`Not Found - ${req.originalUrl}`);
    res.status(404);
    next(error);
};