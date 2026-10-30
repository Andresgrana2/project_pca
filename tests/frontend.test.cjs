const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { pathToFileURL } = require('node:url');

const sourcePath = path.resolve(__dirname, '../statics/js/order.js');
const source = fs.readFileSync(sourcePath, 'utf8');
function makeForm(fetchMock = async () => ({ok:true,json:async()=>({total_value:10})})) {
  const context = { fetch:fetchMock, Intl, Number, String, Object, Error, TypeError };
  const factory = vm.runInNewContext(source + '\norderForm;', context,
    { filename:pathToFileURL(sourcePath).href });
  const form = factory();
  form.form = {weight:'2.5',order_type:'STANDARD',discount:'NEW'};
  form.$nextTick = callback => callback();
  form.$el = {querySelector:()=>({focus(){}})};
  return form;
}

test('validates accepted weight boundaries and client/shipping combinations', () => {
  const form = makeForm();
  for (const weight of ['0.01','5','5.01','20']) {
    for (const order_type of ['STANDARD','EXPRESS']) {
      for (const discount of ['NEW','FREQUENT','VIP']) {
        form.form={weight,order_type,discount}; assert.equal(form.validate(),true);
      }
    }
  }
});
test('rejects blank, nonnumeric, nonfinite and out of range weights', () => {
  const form=makeForm();
  for(const weight of ['', ' ', 'abc', 'Infinity', '-1', '0', '0.009', '20.01', '25']) {
    form.form.weight=weight; assert.equal(form.validate(),false);
    assert.equal(typeof form.errors.weight,'string');
  }
});
test('rejects unknown client and shipping options', () => {
  const form=makeForm(); form.form.order_type='OTHER'; form.form.discount='OTHER';
  assert.equal(form.validate(),false);
  assert.ok(form.errors.order_type); assert.ok(form.errors.discount);
});
test('clears only edited field feedback and prior result', () => {
  const form=makeForm(); form.errors={weight:'wrong',discount:'missing'};
  form.error='old error'; form.result={total_value:12}; form.clearFeedback('weight');
  assert.equal(form.errors.weight,undefined); assert.equal(form.errors.discount,'missing');
  assert.equal(form.error,''); assert.equal(form.result,null);
});
test('does not submit a request while a submission is active', async () => {
  let calls=0;const form=makeForm(async()=>{calls++;}); form.submitting=true;
  await form.submit(); assert.equal(calls,0); assert.equal(form.submitting,true);
});
test('invalid submission focuses the first erroneous field', async () => {
  let calls=0, focused=0;const form=makeForm(async()=>{calls++;});form.form.weight='';
  form.$el={querySelector:selector=>{assert.equal(selector,'.is-invalid');return {focus(){focused++;}};}};
  await form.submit();assert.equal(calls,0);assert.equal(focused,1);assert.equal(form.submitting,false);
});
test('invalid submission tolerates absence of a focus target',async()=>{
  const form=makeForm();form.form.weight='';form.$el={querySelector:()=>null};
  await form.submit();assert.ok(form.errors.weight);
});
test('submits numeric weight and displays a successful quotation', async () => {
  let called=0;const form=makeForm(async(url,options)=>{
    called++; assert.equal(url,'/api/order/new'); assert.equal(options.method,'POST');
    assert.equal(options.headers['Content-Type'],'application/json');
    assert.deepEqual(JSON.parse(options.body),{weight:2.5,order_type:'STANDARD',discount:'NEW'});
    return {ok:true,json:async()=>({total_value:10,discount:'NEW',order_type:'STANDARD'})};
  });
  await form.submit();assert.equal(called,1);assert.equal(form.result.total_value,10);
  assert.equal(form.submitting,false);assert.equal(form.error,'');
});
test('shows API error text and releases submission lock',async()=>{
  const form=makeForm(async()=>({ok:false,json:async()=>({error:'Excede el peso máximo'})}));
  await form.submit();assert.equal(form.error,'Excede el peso máximo');
  assert.equal(form.result,null);assert.equal(form.submitting,false);
});
test('uses fallback feedback for missing or nontext API error',async()=>{
  for(const data of [null,{error:42},{}]) {
    const form=makeForm(async()=>({ok:false,json:async()=>data}));await form.submit();
    assert.match(form.error,/No se pudo calcular/);assert.equal(form.submitting,false);
  }
});
test('handles a response that is not valid JSON',async()=>{
  const form=makeForm(async()=>({ok:false,json:async()=>{throw new SyntaxError('invalid JSON');}}));
  await form.submit();assert.match(form.error,/No se pudo calcular/);
});
test('rejects successful responses without a finite numeric total',async()=>{
  for(const data of [null,{}, {total_value:'10'},{total_value:Infinity}]) {
    const form=makeForm(async()=>({ok:true,json:async()=>data}));await form.submit();
    assert.match(form.error,/respuesta inválida/);assert.equal(form.result,null);
    assert.equal(form.submitting,false);
  }
});
test('shows actionable connection feedback for network failure',async()=>{
  const form=makeForm(async()=>{throw new TypeError('fetch failed');});await form.submit();
  assert.match(form.error,/No se pudo conectar/);assert.equal(form.submitting,false);
});
test('preserves other error messages',async()=>{
  const form=makeForm(async()=>{throw new Error('Unexpected test failure');});await form.submit();
  assert.equal(form.error,'Unexpected test failure');
});
test('formats finite amounts and leaves invalid amounts blank',()=>{
  const form=makeForm();assert.equal(form.formatTotal(1234.5),'1.234,50');
  for(const value of [NaN,Infinity,undefined,null,'12'])assert.equal(form.formatTotal(value),'');
});
