'use client';

const PAYMENT_POLL_INTERVAL_MS = 3000;
const PAYMENT_POLL_ATTEMPTS = 40;

export type PaymentPollingState = {
  timer: number | null;
  orderNo: string | null;
  attempts: number;
  modalOpen: boolean;
  inFlight: boolean;
};

export type OrderStatus = {
  order_no: string;
  status: string;
};

export function createPaymentPollingState(modalOpen: boolean): PaymentPollingState {
  return {
    timer: null,
    orderNo: null,
    attempts: 0,
    modalOpen,
    inFlight: false,
  };
}

export function startOrderPolling(
  orderNo: string,
  polling: PaymentPollingState,
  refreshOrders: () => Promise<unknown> | unknown,
  refreshWallet: VoidFunction,
  onPaid: VoidFunction
) {
  stopOrderPolling(polling);
  polling.orderNo = orderNo;
  polling.attempts = 0;
  if (!polling.modalOpen) {
    return;
  }
  polling.timer = window.setInterval(() => {
    void pollOrderStatus(polling, refreshOrders, refreshWallet, onPaid);
  }, PAYMENT_POLL_INTERVAL_MS);
}

export async function pollOrderStatus(
  polling: PaymentPollingState,
  refreshOrders: () => Promise<unknown> | unknown,
  refreshWallet: VoidFunction,
  onPaid: VoidFunction
) {
  if (!polling.modalOpen || polling.inFlight || !polling.orderNo) {
    return;
  }
  polling.inFlight = true;
  try {
    polling.attempts += 1;
    const paid = await refreshAndMatchOrder(polling.orderNo, refreshOrders, refreshWallet);
    if (paid) {
      onPaid();
      stopOrderPolling(polling);
      return;
    }
    if (polling.attempts >= PAYMENT_POLL_ATTEMPTS) {
      stopOrderPolling(polling);
    }
  } finally {
    polling.inFlight = false;
  }
}

export async function refreshAndMatchOrder(
  orderNo: string,
  refreshOrders: () => Promise<unknown> | unknown,
  refreshWallet: VoidFunction
) {
  const response = await refreshOrders();
  refreshWallet();
  return orderItems(response).some((item) => item.order_no === orderNo && item.status === 'paid');
}

export function stopOrderPolling(polling: PaymentPollingState) {
  if (polling.timer !== null) {
    window.clearInterval(polling.timer);
  }
  polling.timer = null;
  polling.orderNo = null;
  polling.attempts = 0;
}

function orderItems(response: unknown): OrderStatus[] {
  const directItems = pageItems(response);
  if (directItems.length > 0) {
    return directItems;
  }
  if (typeof response === 'object' && response !== null && 'data' in response) {
    return pageItems((response as { data?: unknown }).data);
  }
  return [];
}

function pageItems(response: unknown): OrderStatus[] {
  if (typeof response !== 'object' || response === null || !('items' in response)) {
    return [];
  }
  const items = (response as { items?: unknown }).items;
  if (!Array.isArray(items)) {
    return [];
  }
  return items.filter(isOrderStatus);
}

function isOrderStatus(value: unknown): value is OrderStatus {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as { order_no?: unknown }).order_no === 'string' &&
    typeof (value as { status?: unknown }).status === 'string'
  );
}
