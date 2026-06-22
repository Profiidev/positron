describe('Script Execution', () => {
  describe('Synchronous Script Execution', () => {
    it('should execute simple script', async () => {
      const result = await browser.execute(() => 
        1 + 1
      );
      expect(result).toBe(2);
    });
  });
});
