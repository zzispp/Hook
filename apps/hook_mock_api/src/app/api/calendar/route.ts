import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _events } from 'src/_mock/_event';

// ----------------------------------------------------------------------

export const runtime = 'edge';

type EventType = ReturnType<typeof _events>[number];
let eventsData: Map<string, EventType> = new Map();

function loggerData(action?: string, value?: unknown) {
  logger('[Event] total-events', eventsData.size);
  if (value || action) {
    logger(`[Event] ${action}`, value);
  }
}

function initializeEvents() {
  if (eventsData.size === 0) {
    const events = _events();
    eventsData = new Map(events.map((event) => [event.id, event]));
  }
}

/** **************************************
 * GET - All events
 *************************************** */
export async function GET() {
  try {
    initializeEvents();

    loggerData();

    return response({ events: Array.from(eventsData.values()) }, STATUS.OK);
  } catch (error) {
    return handleError('Event - Get all', error);
  }
}

/** **************************************
 * POST - Create event
 *************************************** */
export async function POST(req: NextRequest) {
  try {
    const { eventData } = await req.json();

    eventsData.set(eventData.id, eventData);

    loggerData('created', eventData);

    return response({ event: eventData }, STATUS.OK);
  } catch (error) {
    return handleError('Event - Create', error);
  }
}

/** **************************************
 * PUT - Update event
 *************************************** */
export async function PUT(req: NextRequest) {
  try {
    const { eventData } = await req.json();

    if (!eventsData.has(eventData.id)) {
      return response({ message: 'Event not found!' }, STATUS.NOT_FOUND);
    }

    const event = eventsData.get(eventData.id);

    // Merge the existing event with the updated data
    const updatedEvent = {
      ...event,
      ...eventData,
    };

    eventsData.set(eventData.id, updatedEvent);

    loggerData('updated', updatedEvent);

    return response({ event: updatedEvent }, STATUS.OK);
  } catch (error) {
    return handleError('Event - Update', error);
  }
}

/** **************************************
 * PATCH - Delete event
 *************************************** */
export async function PATCH(req: NextRequest) {
  try {
    const { eventId } = await req.json();

    if (!eventsData.has(eventId)) {
      return response({ message: 'Event not found!' }, STATUS.NOT_FOUND);
    }

    eventsData.delete(eventId);

    loggerData('deleted', eventId);

    return response({ eventId }, STATUS.OK);
  } catch (error) {
    return handleError('Event - Delete', error);
  }
}
