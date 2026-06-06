<script lang="ts">
  import { Button } from '@profidev/pleiades/components/ui/button';
  import { toast } from '@profidev/pleiades/components/util/general';
</script>

<p>Auth</p>
<Button
  onclick={() => {
    toast.success('Test');
  }}
  class="mt-20"
>
  Test
</Button>
