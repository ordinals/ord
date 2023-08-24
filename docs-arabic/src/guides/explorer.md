 <div dir="rtl">
 
#  متصفح ال ordinals




يتضمن البرنامج الثنائي للأورد  إكسبلورر كتل. نحن نستضيف مثيلًا من  إكسبلورر الكتل على الشبكة الرئيسية على موقع [ordinals.com](https://ordinals.com/)، وعلى شبكة الاختبار signet على موقع [signet.ordinals.com](https://signet.ordinals.com/).


# Running The Explorer تشغيل المتصفح

يمكن تشغيل المتصفح محليا على :</p>
<p><code>ord server</code></p>
<p>لتحديد البورت  <code>--http-port</code> ومن ثم flag:</p>
<p><code>ord server --http-port 8080</code></p>
<p>لاختبار كيف سيبدو inscriptions يمكنك اضافه الامر التالي :</p>
<p><code>ord preview &lt;FILE1&gt; &lt;FILE2&gt; ...</code></p>


## البحث
يمكنك البحث عن عدة أشياء كالتالي 

## Blocks
يمكن البحث عن Blocks عبر الهاش، على سبيل المثال، block genesis :


[000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f](https://ordinals.com/block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f)


## المعاملات Transactions
يمكن البحث عن المعاملات عبر الهاش، على سبيل المثال، معاملة العملات النقدية في block genesis :


[4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b](https://ordinals.com/tx/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b)


## المخرجات Outputs
البحث عن مخرجات المعاملات عبر نقطةالإخراج Outputs، على سبيل المثال، الناتج الوحيد لمعاملة العملات النقدية في block genesis :


[4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b](https://ordinals.com/output/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0)


## ال سات Sats
يمكن البحث عن ال سات Sats عبر العدد الصحيح، وموقعها ضمن إجمالي إمدادات البيتكوين:


[2099994106992659](https://ordinals.com/sat/2099994106992659)


بالعشري، وموقعها في block والإزاحة ضمن تلك Block:


[481824.0](https://ordinals.com/sat/481824.0)


بالزاوية، والدورة، blocks منذ آخر تقسيم  blocks  منذ آخر تعديل للصعوبة، والإزاحة ضمن block:


[1°0′0″0‴](https://ordinals.com/sat/1%C2%B00%E2%80%B20%E2%80%B30%E2%80%B4)


بالاسم، والتمثيل الأساسي الـ 26 باستخدام الأحرف "a" حتى "z":


[ahistorical](https://ordinals.com/sat/ahistorical)


أو بالنسبة المئوية، نسبة إمداد بيتكوين التي تم إصدارها أو سيتم إصدارها عندما يتم تعدينها:


[100%](https://ordinals.com/sat/100%)

