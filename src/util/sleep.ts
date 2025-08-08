export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const waitUntilTrue = (checkFunction: () => boolean, interval = 100): Promise<void> => {
  return new Promise((resolve) => {
    const check = () => {
      if (checkFunction()) {
        resolve();
      } else {
        setTimeout(check, interval);
      }
    };
    check();
  });
}