export enum LocationEnum {
    TH = 8,
    MH = 9,
}

export function newLocationEnum(param?: string): LocationEnum {
    return param === "mh" ? LocationEnum.MH : LocationEnum.TH;
}


const weekdays = [
    ["so", "mo", "di", "mi", "do", "fr", "sa"],
    ["su", "mo", "tu", "we", "th", "fr", "sa"],
    ["sun", "mon", "tue", "wed", "thu", "fri", "sat"],
    ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"]
]

export function parseDay(dayString?: string): number {
    if (!dayString) return -1;
    const day = dayString.toLowerCase();
    return weekdays.find(weekday => weekday.includes(day))?.indexOf(day) || -1;
}