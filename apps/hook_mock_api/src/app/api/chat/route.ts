import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _contacts, _conversations } from 'src/_mock/_chat';

// ----------------------------------------------------------------------

export const runtime = 'edge';

type ConversationType = ReturnType<typeof _conversations>[number];
let conversationsData = new Map<string, ConversationType>();

const ENDPOINTS = {
  CONVERSATIONS: 'conversations',
  CONVERSATION: 'conversation',
  MARK_AS_SEEN: 'mark-as-seen',
  CONTACTS: 'contacts',
};

function loggerData(action?: string, value?: unknown) {
  logger('[Chat] total-conversations', conversationsData.size);
  if (value || action) {
    logger(`[Chat] ${action}`, value);
  }
}

function initializeConversations() {
  if (conversationsData.size === 0) {
    const conversations = _conversations();
    conversationsData = new Map(conversations.map((conv) => [conv.id, conv]));
  }
}

/** **************************************
 * GET - Handle actions based on the endpoint
 *************************************** */
export async function GET(req: NextRequest) {
  try {
    const { searchParams } = req.nextUrl;
    const endpoint = searchParams.get('endpoint');

    switch (endpoint) {
      case ENDPOINTS.CONVERSATIONS:
        return getConversations();
      case ENDPOINTS.CONVERSATION:
        return getConversation(req);
      case ENDPOINTS.MARK_AS_SEEN:
        return markAsSeen(req);
      case ENDPOINTS.CONTACTS:
        return getContacts();
      default:
        return response({ message: 'Endpoint not found!' }, STATUS.NOT_FOUND);
    }
  } catch (error) {
    return handleError(`Chat - Get request`, error);
  }
}

/** **************************************
 * POST - Create conversation
 *************************************** */
export async function POST(req: NextRequest) {
  try {
    const { conversationData } = await req.json();

    conversationsData.set(conversationData.id, conversationData);

    loggerData('created-conversation', conversationData.id);

    return response({ conversation: conversationData }, STATUS.OK);
  } catch (error) {
    return handleError('Chat - Create conversation', error);
  }
}

/** **************************************
 * PUT - Update conversation
 *************************************** */
export async function PUT(req: NextRequest) {
  try {
    const { conversationId, messageData } = await req.json();

    const conversation = conversationsData.get(conversationId);

    if (!conversation) {
      return response({ message: 'Conversation not found!' }, STATUS.NOT_FOUND);
    }

    const updatedConversation = {
      ...conversation,
      messages: [...conversation.messages, messageData],
    };

    conversationsData.set(conversationId, updatedConversation);

    loggerData('updated-conversation', conversationId);

    return response({ conversation: updatedConversation }, STATUS.OK);
  } catch (error) {
    return handleError('Chat - Update conversation', error);
  }
}

/** **************************************
 * GET - Contact list
 *************************************** */
async function getContacts() {
  return response({ contacts: _contacts() }, STATUS.OK);
}

/** **************************************
 * GET - Conversation list
 *************************************** */
async function getConversations() {
  try {
    initializeConversations();

    loggerData();

    return response({ conversations: Array.from(conversationsData.values()) }, STATUS.OK);
  } catch (error) {
    return handleError('Chat - Get conversations', error);
  }
}

/** **************************************
 * GET - Conversation
 *************************************** */
async function getConversation(req: NextRequest) {
  initializeConversations(); // Fix when the conversationsData is empty

  const { searchParams } = req.nextUrl;
  const conversationId = searchParams.get('conversationId');

  if (!conversationId) {
    return response({ message: 'Missing conversation id!' }, STATUS.BAD_REQUEST);
  }

  const conversation = conversationsData.get(conversationId);

  if (!conversation) {
    return response({ message: 'Conversation not found!' }, STATUS.NOT_FOUND);
  }

  loggerData('get-conversation', conversation.id);

  return response({ conversation }, STATUS.OK);
}

/** **************************************
 * PUT - Mark conversation as seen
 *************************************** */
async function markAsSeen(req: NextRequest) {
  const { searchParams } = req.nextUrl;
  const conversationId = searchParams.get('conversationId');

  if (!conversationId) {
    return response({ message: 'Missing conversation id!' }, STATUS.BAD_REQUEST);
  }

  const conversation = conversationsData.get(conversationId);

  if (!conversation) {
    return response({ message: 'Conversation not found!' }, STATUS.NOT_FOUND);
  }

  const updatedConversation = {
    ...conversation,
    unreadCount: 0,
  };
  conversationsData.set(conversationId, updatedConversation);

  loggerData('conversation-marked-as-seen', conversation.id);

  return response({ conversationId }, STATUS.OK);
}
