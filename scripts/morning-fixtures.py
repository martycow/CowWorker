from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parents[1]
out = root / 'output' / 'verification'
out.mkdir(parents=True, exist_ok=True)
fixtures = root / 'backend' / 'tests' / 'fixtures'
fixtures.mkdir(parents=True, exist_ok=True)
font = ImageFont.truetype(str(root / 'backend/assets/NotoSans-Regular.ttf'), 32)
pages = [
    ['Title: Senior Software Engineer', 'Company: Morning Example', 'Location: Remote', '', 'Requirements', 'Rust and TypeScript experience', 'Clear written communication'],
    ['Responsibilities', 'Build reliable desktop applications', 'Work with designers and users', '', 'Salary: 120000 USD per year', 'Employment type: Full time'],
]
for index, lines in enumerate(pages, start=1):
    image = Image.new('RGB', (1200, 620), 'white')
    draw = ImageDraw.Draw(image)
    for row, text in enumerate(lines):
        draw.text((55, 45 + row * 65), text, font=font, fill='#17202a')
    image.save(out / f'vacancy-screen-{index}.png')
    image.save(fixtures / f'vacancy-screen-{index}.png')
Image.open(fixtures / 'vacancy-screen-1.png').convert('RGB').save(fixtures / 'scanned-vacancy.pdf', 'PDF', resolution=150)

from docx import Document
from reportlab.pdfgen import canvas
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont

resume = '''Jane Doe
Software Engineer
jane@example.test | Seattle, WA

SUMMARY
Software engineer with experience building reliable desktop applications.

EXPERIENCE
Example Studio | Software Engineer | 2022-2026
Built Rust and TypeScript applications with a small product team.
Improved accessibility and added automated recovery checks.
Collaborated with designers and users to simplify document workflows.

EDUCATION
Example University | Bachelor of Computer Science | 2022

SKILLS
Rust, TypeScript, React, SQLite, automated testing
'''
doc = Document()
for line in resume.splitlines():
    doc.add_paragraph(line)
doc.save(fixtures / 'old-resume.docx')
pdfmetrics.registerFont(TTFont('NotoSans', str(root / 'backend/assets/NotoSans-Regular.ttf')))
pdf = canvas.Canvas(str(fixtures / 'old-resume.pdf'), pagesize=(612, 792))
text = pdf.beginText(54, 738)
text.setFont('NotoSans', 11)
text.setLeading(18)
for line in resume.splitlines():
    text.textLine(line)
pdf.drawText(text)
pdf.save()
