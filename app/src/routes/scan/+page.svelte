<script lang="ts">
  import { goto } from '$app/navigation';
  import { toast } from '@profidev/pleiades/components/util/general';
  import { Format } from '@tauri-apps/plugin-barcode-scanner';
  import { onDestroy, onMount } from 'svelte';

  onMount(async () => {
    const { checkPermissions, requestPermissions, scan, cancel } =
      await import('@tauri-apps/plugin-barcode-scanner');
    const permissions = await checkPermissions();
    if (permissions !== 'granted') {
      if ((await requestPermissions()) !== 'granted') {
        toast.error('Permission denied');
        goto('/');
        return;
      }
    }

    document.body.classList.add('bg-transparent');
    document.body.classList.add('pt-0!');

    try {
      const result = await scan({ formats: [Format.QRCode], windowed: true });
      const url = new URL(result.content);
      const code = url.searchParams.get('code');
      toast.success(`Scanned code: ${code}`);
      goto(`/login?code=${code}`);
    } catch {
      toast.error('Failed to scan QR code');
    }

    document.body.classList.remove('bg-transparent');
    document.body.classList.remove('pt-0!');
  });

  onDestroy(async () => {
    const { cancel } = await import('@tauri-apps/plugin-barcode-scanner');
    cancel();
  });
</script>

<div class="relative h-screen w-full overflow-hidden">
  <div class="pointer-events-none absolute inset-0 flex justify-center">
    <div
      class="border-primary relative mt-40 h-64 w-64 rounded-3xl border-4 shadow-[0_0_0_9999px_rgba(0,0,0,0.8)]"
    ></div>
  </div>
</div>
