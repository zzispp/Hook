import type { PageResponse } from './rbac';

export type AnnouncementType = 'info' | 'warning' | 'maintenance' | 'important';

export type Announcement = {
  id: string;
  title: string;
  content_markdown: string;
  announcement_type: AnnouncementType;
  pinned: boolean;
  priority: number;
  enabled: boolean;
  created_by: string;
  updated_by: string;
  created_at: string;
  updated_at: string;
};

export type AnnouncementInput = {
  title: string;
  content_markdown: string;
  announcement_type: AnnouncementType;
  pinned: boolean;
  priority: number;
  enabled: boolean;
};

export type TicketStatus = 'open' | 'in_progress' | 'waiting_user' | 'resolved' | 'closed';

export type TicketPriority = 'normal' | 'high' | 'urgent';

export type SupportTicket = {
  id: string;
  user_id: string;
  subject: string;
  contact_email: string;
  status: TicketStatus;
  priority: TicketPriority;
  last_message_at: string;
  last_message_sender_role: 'user' | 'admin';
  last_user_activity_at?: string | null;
  last_admin_activity_at?: string | null;
  created_at: string;
  updated_at: string;
};

export type SupportTicketMessage = {
  id: string;
  ticket_id: string;
  sender_user_id: string;
  sender_role: 'user' | 'admin';
  message_kind: string;
  body_markdown: string;
  created_at: string;
};

export type SupportTicketEmailEvent = {
  id: string;
  ticket_id: string;
  message_id?: string | null;
  recipient_email: string;
  subject: string;
  status: 'sent' | 'failed' | 'disabled';
  error_message?: string | null;
  created_at: string;
};

export type SupportTicketDetail = {
  ticket: SupportTicket;
  messages: SupportTicketMessage[];
  email_events: SupportTicketEmailEvent[];
};

export type SupportTicketMutationResponse = {
  ticket: SupportTicket;
  message?: SupportTicketMessage | null;
  email_delivery: {
    status: 'sent' | 'failed' | 'disabled';
    error_code?: string | null;
    error_message?: string | null;
  };
};

export type NotificationItem = {
  source_type: 'announcement' | 'ticket' | 'provider_quick_import_sync';
  source_id: string;
  title: string;
  description?: string | null;
  category: string;
  is_unread: boolean;
  created_at: string;
  link_path: string;
};

export type OperationsPage<T> = PageResponse<T>;
