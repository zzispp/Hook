/**
 * Bind multiple event listeners to a target.
 * https://github.com/alexreardon/bind-event-listener/tree/master
 */

type UnbindFn = () => void;

type UnknownFunction = (...args: any[]) => any;

type InferEventType<TTarget> = TTarget extends {
  addEventListener(type: infer P, ...args: any): void;
  addEventListener(type: infer P2, ...args: any): void;
}
  ? P & string
  : never;

type InferEvent<TTarget, TType extends string> = `on${TType}` extends keyof TTarget
  ? Parameters<Extract<TTarget[`on${TType}`], UnknownFunction>>[0]
  : Event;

type ListenerObject<TEvent extends Event> = {
  handleEvent(this: ListenerObject<TEvent>, event: TEvent): void;
};

type Listener<TTarget extends EventTarget, TType extends string> =
  | ListenerObject<InferEvent<TTarget, TType>>
  | { (this: TTarget, ev: InferEvent<TTarget, TType>): void };

type Binding<TTarget extends EventTarget = EventTarget, TType extends string = string> = {
  type: TType;
  listener: Listener<TTarget, TType>;
  options?: boolean | AddEventListenerOptions;
};

// ----------------------------------------------------------------------

function toOptions(value?: boolean | AddEventListenerOptions): AddEventListenerOptions | undefined {
  if (typeof value === 'undefined') {
    return undefined;
  }

  if (typeof value === 'boolean') {
    return {
      capture: value,
    };
  }

  return value;
}

function getBinding(original: Binding, sharedOptions?: boolean | AddEventListenerOptions): Binding {
  if (sharedOptions == null) {
    return original;
  }

  const binding: Binding = {
    ...original,
    options: {
      ...toOptions(sharedOptions),
      ...toOptions(original.options),
    },
  };
  return binding;
}

/**
 * Binds a single event listener to a target and returns an unbind function
 */
export function bind<
  TTarget extends EventTarget,
  TType extends InferEventType<TTarget> | (string & {}),
>(target: TTarget, { type, listener, options }: Binding<TTarget, TType>): UnbindFn {
  target.addEventListener(type, listener, options);

  return function unbind() {
    target.removeEventListener(type, listener, options);
  };
}

/**
 * Binds multiple event listeners to a target and returns a function to unbind them all
 */
export function bindAll<
  TTarget extends EventTarget,
  TTypes extends ReadonlyArray<InferEventType<TTarget> | (string & {})>,
>(
  target: TTarget,
  bindings: [
    ...{
      [K in keyof TTypes]: {
        type: TTypes[K];
        listener: Listener<TTarget, TTypes[K] & string>;
        options?: boolean | AddEventListenerOptions;
      };
    },
  ],
  sharedOptions?: boolean | AddEventListenerOptions
): UnbindFn {
  const unbinds: UnbindFn[] = bindings.map((original) => {
    const binding: Binding = getBinding(original as never, sharedOptions);
    return bind(target, binding);
  });

  return function unbindAll() {
    unbinds.forEach((unbind) => unbind());
  };
}
